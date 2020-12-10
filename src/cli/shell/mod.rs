use super::QueryOutput;
use crate::{config::Config, utils::read_file};
use anyhow::Context;
use colored::*;
use highlight::{MonoKaiSchema, Schema};
use rustyline::error::ReadlineError;
use structopt::StructOpt;

mod helper;
mod highlight;

#[derive(Debug)]
pub struct Shell;

#[derive(Debug, StructOpt)]
#[structopt(name = "DBuiltin", about = "DCli 内建命令")]
pub enum BuiltIn {
    #[structopt(name = "%exit", about = "退出 shell")]
    Exit,
    #[structopt(name = "%help", about = "打印帮助信息")]
    Help,
    #[structopt(name = "%his", about = "查看历史")]
    His,
    #[structopt(name = "%run", about = "运行 SQL 文件")]
    Run {
        /// 文件路径
        path: String,
    },
}

impl Shell {
    pub async fn run(config: &mut Config, profile: &str) -> anyhow::Result<()> {
        let profile = config.try_get_profile(profile)?;
        let history = profile.load_or_create_history()?;
        let mut pool = crate::mysql::connect(&profile).await?;
        let mut rl = helper::get_editor(&mut pool).await?;
        let mut count: usize = 1;
        rl.load_history(&history)
            .with_context(|| "无法载入历史文件.")?;
        loop {
            let p = format!("[{}]: ", count)
                .color(MonoKaiSchema::green())
                .to_string();
            rl.helper_mut().with_context(|| "无 helper")?.colored_prompt = p.clone();
            let input = rl.readline(&p);
            match input {
                Ok(line) => {
                    if !line.is_empty() {
                        match Shell::take_builtin(&line) {
                            Ok(maybe_builtin) => {
                                if let Some(builtin) = maybe_builtin {
                                    match builtin {
                                        BuiltIn::Exit => {
                                            println!("Exit...");
                                            break;
                                        }
                                        BuiltIn::Help => {
                                            BuiltIn::clap().print_help().unwrap();
                                        }
                                        BuiltIn::His => {
                                            rl.history()
                                                .iter()
                                                .enumerate()
                                                .for_each(|(i, h)| println!("{} {}", i, h));
                                        }
                                        BuiltIn::Run { path } => match read_file(&path) {
                                            Ok(content) => {
                                                for sql in content.split(";") {
                                                    if !sql.is_empty() {
                                                        let output: QueryOutput = sqlx::query(sql)
                                                            .fetch_all(&pool)
                                                            .await?
                                                            .into();
                                                        println!(
                                                            "{}",
                                                            output.to_print_table(&config)
                                                        );
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!("{:?}", e);
                                            }
                                        },
                                    }
                                    rl.add_history_entry(line.as_str());
                                } else {
                                    match sqlx::query(&line).fetch_all(&pool).await {
                                        Ok(resp) => {
                                            let output: QueryOutput = resp.into();
                                            println!("{}", output.to_print_table(config));
                                            rl.add_history_entry(line.as_str());
                                        }
                                        Err(e) => {
                                            println!("Server Err: {}", e)
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("{}", e)
                            }
                        }
                    } else {
                        println!("");
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    println!("使用 %exit 退出.");
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
            count += 1;
        }
        pool.close().await;
        rl.append_history(&history).unwrap();
        Ok(())
    }

    fn take_builtin(line: &str) -> anyhow::Result<Option<BuiltIn>> {
        if line.starts_with("%") {
            let builtin =
                BuiltIn::from_iter_safe(format!("builtin {}", line).split_ascii_whitespace())
                    .map_err(|mut e| {
                        e.message = e
                            .message
                            .replace("builtin <SUBCOMMAND>", "%<cmd>")
                            .replace("builtin --", "");
                        e
                    })?;
            Ok(Some(builtin))
        } else {
            Ok(None)
        }
    }
}

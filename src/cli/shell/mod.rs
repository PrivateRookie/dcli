use crate::{config::Config, utils::read_file};
use crate::{fl, mysql::Session};
use anyhow::Context;
use colored::*;
use highlight::{MonoKaiSchema, Schema};
use rustyline::error::ReadlineError;
use structopt::StructOpt;

mod helper;
mod highlight;

#[derive(Debug)]
pub struct Shell;

#[cfg_attr(feature = "zh-CN", doc = "DCli 内建命令")]
#[cfg_attr(feature = "en-US", doc = "Dcli builtin commands")]
#[derive(Debug, StructOpt)]
#[structopt(name = "DBuiltin")]
pub enum BuiltIn {
    #[cfg_attr(feature = "zh-CN", doc = "退出 shell")]
    #[cfg_attr(feature = "en-US", doc = "exit shell")]
    #[structopt(name = "%exit")]
    Exit,

    #[cfg_attr(feature = "zh-CN", doc = "打印帮助信息")]
    #[cfg_attr(feature = "en-US", doc = "print help message")]
    #[structopt(name = "%help")]
    Help,

    #[cfg_attr(feature = "zh-CN", doc = "查看历史")]
    #[cfg_attr(feature = "en-US", doc = "list history")]
    #[structopt(name = "%his")]
    His,

    #[cfg_attr(feature = "zh-CN", doc = "运行 SQL 文件")]
    #[cfg_attr(feature = "en-US", doc = "exec SQL file")]
    #[structopt(name = "%run")]
    Run {
        #[cfg_attr(feature = "zh-CN", doc = "文件路径")]
        #[cfg_attr(feature = "en-US", doc = "path file")]
        path: String,
    },
}

impl Shell {
    pub async fn run(config: &mut Config, profile: &str) -> anyhow::Result<()> {
        let profile = config.try_get_profile(profile)?;
        let history = profile.load_or_create_history()?;
        let mut session = Session::connect_with(&profile).await?;
        let mut rl = helper::get_editor(&mut session).await?;
        let mut count: usize = 1;
        rl.load_history(&history)
            .with_context(|| fl!("load-his-failed"))?;
        loop {
            let p = format!("[{}]: ", count)
                .color(MonoKaiSchema::green())
                .to_string();
            rl.helper_mut().unwrap().colored_prompt = p.clone();
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
                                                for sql in content.split(';') {
                                                    if !sql.is_empty() {
                                                        let output = session.query(sql).await?;
                                                        output.to_print_table(&config, false);
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
                                    match session.query(&line).await {
                                        Ok(output) => {
                                            output.to_print_table(config, false);
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
                        println!();
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    println!("{}", fl!("exit-info"));
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
            count += 1;
        }
        session.close().await;
        rl.append_history(&history).unwrap();
        Ok(())
    }

    fn take_builtin(line: &str) -> anyhow::Result<Option<BuiltIn>> {
        if line.starts_with('%') {
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

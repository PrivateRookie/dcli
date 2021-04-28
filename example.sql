-- param username = "rookie" // 用户名, "//" 之后的内容均视为注释
-- param age = 18 // 用户最小年龄
-- param score = 60.0 // 最低分数
-- param school // 学校
-- param addrs // 数组输入 [int] 为不定长数组

SELECT user, age, score, schoole, addr
FROM students
where user=@username AND age >= @age AND score >= @score AND school in @school AND add in @addrs;

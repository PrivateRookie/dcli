pub const KEYWORDS: [&str; 389] = [
    "ABS",
    "ACTION",
    "ADD",
    "ALL",
    "ALLOCATE",
    "ALTER",
    "AND",
    "ANY",
    "APPLY",
    "ARE",
    "ARRAY",
    "ARRAY_AGG",
    "ARRAY_MAX_CARDINALITY",
    "AS",
    "ASC",
    "ASENSITIVE",
    "ASSERT",
    "ASYMMETRIC",
    "AT",
    "ATOMIC",
    "AUTHORIZATION",
    "AVG",
    "AVRO",
    "BEGIN",
    "BEGIN_FRAME",
    "BEGIN_PARTITION",
    "BETWEEN",
    "BIGINT",
    "BINARY",
    "BLOB",
    "BOOLEAN",
    "BOTH",
    "BY",
    "BYTEA",
    "CALL",
    "CALLED",
    "CARDINALITY",
    "CASCADE",
    "CASCADED",
    "CASE",
    "CAST",
    "CEIL",
    "CEILING",
    "CHAIN",
    "CHAR",
    "CHARACTER",
    "CHARACTER_LENGTH",
    "CHAR_LENGTH",
    "CHECK",
    "CLOB",
    "CLOSE",
    "COALESCE",
    "COLLATE",
    "COLLECT",
    "COLUMN",
    "COLUMNS",
    "COMMIT",
    "COMMITTED",
    "CONDITION",
    "CONNECT",
    "CONSTRAINT",
    "CONTAINS",
    "CONVERT",
    "COPY",
    "CORR",
    "CORRESPONDING",
    "COUNT",
    "COVAR_POP",
    "COVAR_SAMP",
    "CREATE",
    "CROSS",
    "CSV",
    "CUBE",
    "CUME_DIST",
    "CURRENT",
    "CURRENT_CATALOG",
    "CURRENT_DATE",
    "CURRENT_DEFAULT_TRANSFORM_GROUP",
    "CURRENT_PATH",
    "CURRENT_ROLE",
    "CURRENT_ROW",
    "CURRENT_SCHEMA",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "CURRENT_TRANSFORM_GROUP_FOR_TYPE",
    "CURRENT_USER",
    "CURSOR",
    "CYCLE",
    "DATE",
    "DAY",
    "DEALLOCATE",
    "DEC",
    "DECIMAL",
    "DECLARE",
    "DEFAULT",
    "DELETE",
    "DENSE_RANK",
    "DEREF",
    "DESC",
    "DESCRIBE",
    "DETERMINISTIC",
    "DISCONNECT",
    "DISTINCT",
    "DOUBLE",
    "DROP",
    "DYNAMIC",
    "EACH",
    "ELEMENT",
    "ELSE",
    "END",
    "END_EXEC",
    "END_FRAME",
    "END_PARTITION",
    "EQUALS",
    "ERROR",
    "ESCAPE",
    "EVERY",
    "EXCEPT",
    "EXEC",
    "EXECUTE",
    "EXISTS",
    "EXP",
    "EXTENDED",
    "EXTERNAL",
    "EXTRACT",
    "FALSE",
    "FETCH",
    "FIELDS",
    "FILTER",
    "FIRST",
    "FIRST_VALUE",
    "FLOAT",
    "FLOOR",
    "FOLLOWING",
    "FOR",
    "FOREIGN",
    "FRAME_ROW",
    "FREE",
    "FROM",
    "FULL",
    "FUNCTION",
    "FUSION",
    "GET",
    "GLOBAL",
    "GRANT",
    "GROUP",
    "GROUPING",
    "GROUPS",
    "HAVING",
    "HEADER",
    "HOLD",
    "HOUR",
    "IDENTITY",
    "IF",
    "IN",
    "INDEX",
    "INDICATOR",
    "INNER",
    "INOUT",
    "INSENSITIVE",
    "INSERT",
    "INT",
    "INTEGER",
    "INTERSECT",
    "INTERSECTION",
    "INTERVAL",
    "INTO",
    "IS",
    "ISOLATION",
    "JOIN",
    "JSONFILE",
    "KEY",
    "LAG",
    "LANGUAGE",
    "LARGE",
    "LAST",
    "LAST_VALUE",
    "LATERAL",
    "LEAD",
    "LEADING",
    "LEFT",
    "LEVEL",
    "LIKE",
    "LIKE_REGEX",
    "LIMIT",
    "LISTAGG",
    "LN",
    "LOCAL",
    "LOCALTIME",
    "LOCALTIMESTAMP",
    "LOCATION",
    "LOWER",
    "MATCH",
    "MATERIALIZED",
    "MAX",
    "MEMBER",
    "MERGE",
    "METHOD",
    "MIN",
    "MINUTE",
    "MOD",
    "MODIFIES",
    "MODULE",
    "MONTH",
    "MULTISET",
    "NATIONAL",
    "NATURAL",
    "NCHAR",
    "NCLOB",
    "NEW",
    "NEXT",
    "NO",
    "NONE",
    "NORMALIZE",
    "NOT",
    "NTH_VALUE",
    "NTILE",
    "NULL",
    "NULLIF",
    "NULLS",
    "NUMERIC",
    "OBJECT",
    "OCCURRENCES_REGEX",
    "OCTET_LENGTH",
    "OF",
    "OFFSET",
    "OLD",
    "ON",
    "ONLY",
    "OPEN",
    "OR",
    "ORC",
    "ORDER",
    "OUT",
    "OUTER",
    "OVER",
    "OVERFLOW",
    "OVERLAPS",
    "OVERLAY",
    "PARAMETER",
    "PARQUET",
    "PARTITION",
    "PERCENT",
    "PERCENTILE_CONT",
    "PERCENTILE_DISC",
    "PERCENT_RANK",
    "PERIOD",
    "PORTION",
    "POSITION",
    "POSITION_REGEX",
    "POWER",
    "PRECEDES",
    "PRECEDING",
    "PRECISION",
    "PREPARE",
    "PRIMARY",
    "PROCEDURE",
    "RANGE",
    "RANK",
    "RCFILE",
    "READ",
    "READS",
    "REAL",
    "RECURSIVE",
    "REF",
    "REFERENCES",
    "REFERENCING",
    "REGCLASS",
    "REGR_AVGX",
    "REGR_AVGY",
    "REGR_COUNT",
    "REGR_INTERCEPT",
    "REGR_R2",
    "REGR_SLOPE",
    "REGR_SXX",
    "REGR_SXY",
    "REGR_SYY",
    "RELEASE",
    "RENAME",
    "REPEATABLE",
    "RESTRICT",
    "RESULT",
    "RETURN",
    "RETURNS",
    "REVOKE",
    "RIGHT",
    "ROLLBACK",
    "ROLLUP",
    "ROW",
    "ROWID",
    "ROWS",
    "ROW_NUMBER",
    "SAVEPOINT",
    "SCHEMA",
    "SCOPE",
    "SCROLL",
    "SEARCH",
    "SECOND",
    "SELECT",
    "SENSITIVE",
    "SEQUENCEFILE",
    "SERIALIZABLE",
    "SESSION",
    "SESSION_USER",
    "SET",
    "SHOW",
    "SIMILAR",
    "SMALLINT",
    "SOME",
    "SPECIFIC",
    "SPECIFICTYPE",
    "SQL",
    "SQLEXCEPTION",
    "SQLSTATE",
    "SQLWARNING",
    "SQRT",
    "START",
    "STATIC",
    "STDDEV_POP",
    "STDDEV_SAMP",
    "STDIN",
    "STORED",
    "SUBMULTISET",
    "SUBSTRING",
    "SUBSTRING_REGEX",
    "SUCCEEDS",
    "SUM",
    "SYMMETRIC",
    "SYSTEM",
    "SYSTEM_TIME",
    "SYSTEM_USER",
    "TABLE",
    "TABLESAMPLE",
    "TEXT",
    "TEXTFILE",
    "THEN",
    "TIES",
    "TIME",
    "TIMESTAMP",
    "TIMEZONE_HOUR",
    "TIMEZONE_MINUTE",
    "TO",
    "TOP",
    "TRAILING",
    "TRANSACTION",
    "TRANSLATE",
    "TRANSLATE_REGEX",
    "TRANSLATION",
    "TREAT",
    "TRIGGER",
    "TRIM",
    "TRIM_ARRAY",
    "TRUE",
    "TRUNCATE",
    "UESCAPE",
    "UNBOUNDED",
    "UNCOMMITTED",
    "UNION",
    "UNIQUE",
    "UNKNOWN",
    "UNNEST",
    "UPDATE",
    "UPPER",
    "USER",
    "USING",
    "UUID",
    "VALUE",
    "VALUES",
    "VALUE_OF",
    "VARBINARY",
    "VARCHAR",
    "VARYING",
    "VAR_POP",
    "VAR_SAMP",
    "VERSIONING",
    "VIEW",
    "VIRTUAL",
    "WHEN",
    "WHENEVER",
    "WHERE",
    "WIDTH_BUCKET",
    "WINDOW",
    "WITH",
    "WITHIN",
    "WITHOUT",
    "WORK",
    "WRITE",
    "YEAR",
    "ZONE",
];
pub const SCHEMA_TABLE: &str = "information_schema";

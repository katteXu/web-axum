CREATE TABLE IF NOT EXISTS user
(
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    role_id INTEGER DEFAULT NULL
);


-- 域名列表
CREATE TABLE IF NOT EXISTS domain
(
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    -- 域名
    domain_name VARCHAR(255) NOT NULL UNIQUE,
    -- 状态
    domain_status VARCHAR(255) DEFAULT NULL,
    -- 建站年龄
    domain_age INTEGER DEFAULT NULL,
    -- 记录数
    order_no INTEGER DEFAULT NULL,
    -- 语言
    language VARCHAR(255) DEFAULT NULL,
    -- 标题
    title VARCHAR(255) DEFAULT NULL,
    -- 评分
    score INTEGER DEFAULT NULL,
    -- DNS
    dns VARCHAR(255) DEFAULT NULL,
    -- 注册商
    registrar_name VARCHAR(255) DEFAULT NULL,
    registrar_address VARCHAR(255) DEFAULT NULL,
    registrar_by VARCHAR(255) DEFAULT NULL,
    registrar_at TIMESTAMP DEFAULT NULL,
    -- 到期时间
    expire_at TIMESTAMP DEFAULT NULL,
    -- email
    email VARCHAR(255) DEFAULT NULL,
    -- 备案
    record_name VARCHAR(255) DEFAULT NULL,
    record_no VARCHAR(255) DEFAULT NULL,
    record_status VARCHAR(255) DEFAULT NULL,
    record_at TIMESTAMP DEFAULT NULL,
    record_main_body VARCHAR(255) DEFAULT NULL,
    record_type VARCHAR(255) DEFAULT NULL
);

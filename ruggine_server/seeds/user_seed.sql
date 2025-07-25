-- 1. CREA DATABASE
CREATE DATABASE IF NOT EXISTS ruggine CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
CREATE DATABASE IF NOT EXISTS ruggine_test CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;

-- 2. CREA TABELLA IN ruggine
USE ruggine;

-- ⚠️ Drop se esiste già
DROP TABLE IF EXISTS `user`;

CREATE TABLE `user` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `first_name` VARCHAR(255) NOT NULL,
    `last_name` VARCHAR(255) NOT NULL,
    `username` VARCHAR(255) NOT NULL,
    `email` VARCHAR(255) NOT NULL,
    `password` VARCHAR(255) COLLATE utf8mb4_bin NOT NULL,
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    `is_active` TINYINT(1) NOT NULL DEFAULT 1,
    PRIMARY KEY (`id`),
    UNIQUE KEY `username` (`username`),
    UNIQUE KEY `email` (`email`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Dati iniziali
INSERT INTO `user` (first_name, last_name, username, email, password, is_active) VALUES
('Mario', 'Rossi', 'mariorossi', 'mario.rossi@example.com', '$2b$04$somethinghashed', 1),
('Luigi', 'Verdi', 'luigiverdi', 'luigi.verdi@example.com', '$2b$04$somethinghashed', 1);

-- 3. CREA TABELLA IN ruggine_test
USE ruggine_test;

-- ⚠️ Drop se esiste già
DROP TABLE IF EXISTS `user`;

CREATE TABLE `user` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `first_name` VARCHAR(255) NOT NULL,
    `last_name` VARCHAR(255) NOT NULL,
    `username` VARCHAR(255) NOT NULL,
    `email` VARCHAR(255) NOT NULL,
    `password` VARCHAR(255) COLLATE utf8mb4_bin NOT NULL,
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    `is_active` TINYINT(1) NOT NULL DEFAULT 1,
    PRIMARY KEY (`id`),
    UNIQUE KEY `username` (`username`),
    UNIQUE KEY `email` (`email`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Dato iniziale per test
INSERT INTO `user` (first_name, last_name, username, email, password, is_active) VALUES
('Test', 'User', 'testuser', 'test.user@example.com', '$2b$04$somethinghashed', 1);

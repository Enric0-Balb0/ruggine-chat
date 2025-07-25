-- 1. CREA DATABASE
CREATE DATABASE IF NOT EXISTS ruggine CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
CREATE DATABASE IF NOT EXISTS ruggine_test CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;

-- 2. CREA UTENTI E ASSEGNA PERMESSI
CREATE USER IF NOT EXISTS 'admin'@'localhost' IDENTIFIED BY 'pwd';
GRANT ALL PRIVILEGES ON ruggine.* TO 'admin'@'localhost';

CREATE USER IF NOT EXISTS 'testuser'@'localhost' IDENTIFIED BY 'testpass';
GRANT ALL PRIVILEGES ON ruggine_test.* TO 'testuser'@'localhost';

FLUSH PRIVILEGES;

-- 3. CREA TABELLA IN ruggine
USE ruggine;

CREATE TABLE IF NOT EXISTS `user` (
                                      `id` INT NOT NULL AUTO_INCREMENT,
                                      `first_name` VARCHAR(255) NOT NULL,
    `last_name` VARCHAR(255) NOT NULL,
    `username` VARCHAR(255) NOT NULL,
    `email` VARCHAR(255) NOT NULL,
    `password` VARCHAR(255) COLLATE utf8mb4_bin NOT NULL,
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP NULL DEFAULT NULL ON UPDATE CURRENT_TIMESTAMP,
    `is_active` TINYINT(1) NOT NULL DEFAULT '0',
    PRIMARY KEY (`id`),
    UNIQUE KEY `username` (`username`),
    UNIQUE KEY `email` (`email`)
    ) ENGINE=InnoDB AUTO_INCREMENT=1 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

INSERT INTO `user` (first_name, last_name, username, email, password, is_active) VALUES
                                                                                     ('Mario', 'Rossi', 'mariorossi', 'mario.rossi@example.com', '$2b$04$somethinghashed', 1),
                                                                                     ('Luigi', 'Verdi', 'luigiverdi', 'luigi.verdi@example.com', '$2b$04$somethinghashed', 1);

-- 4. CREA TABELLA IN ruggine_test
USE ruggine_test;

CREATE TABLE IF NOT EXISTS `user` (
                                      `id` INT NOT NULL AUTO_INCREMENT,
                                      `first_name` VARCHAR(255) NOT NULL,
    `last_name` VARCHAR(255) NOT NULL,
    `username` VARCHAR(255) NOT NULL,
    `email` VARCHAR(255) NOT NULL,
    `password` VARCHAR(255) COLLATE utf8mb4_bin NOT NULL,
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP NULL DEFAULT NULL ON UPDATE CURRENT_TIMESTAMP,
    `is_active` TINYINT(1) NOT NULL DEFAULT '0',
    PRIMARY KEY (`id`),
    UNIQUE KEY `username` (`username`),
    UNIQUE KEY `email` (`email`)
    ) ENGINE=InnoDB AUTO_INCREMENT=1 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

INSERT INTO `user` (first_name, last_name, username, email, password, is_active) VALUES
    ('Test', 'User', 'testuser', 'test.user@example.com', '$2b$04$somethinghashed', 1);

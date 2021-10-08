---
title: database
---

# RuokaDB

This file documents the structure of the "RuokaDB" -database

```sql
CREATE DATABASE RuokaDB;
USE RuokaDB;

CREATE TABLE Ruoat (
	RuokaID int NOT NULL AUTO_INCREMENT,
	RuokaName TINYTEXT NOT NULL,
	ImageName TINYTEXT,
	PRIMARY KEY (RuokaID)
);

CREATE TABLE Arvostelut (
	RuokaID int NOT NULL,
	KayttajaID varchar(18) NOT NULL,
	Arvosana int NOT NULL,
	FOREIGN KEY (RuokaID) REFERENCES Ruoat(RuokaID)
);

CREATE TABLE Ruokalista (
	PVM DATE NOT NULL,
	RuokaID int NOT NULL,
	KokoRuoka TINYTEXT NOT NULL,
	FOREIGN KEY (RuokaID) REFERENCES Ruoat(RUOKAID)
);
```

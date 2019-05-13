-- This file should undo anything in `up.sql`
drop table if exists balancer;
drop table if exists cacr;
drop table if exists cmtr;
drop function if exists maintain_updated_atr();
drop index if exists balancer_iban;
drop index if exists uuid_cacr;
drop index if exists uuid_cmtr;
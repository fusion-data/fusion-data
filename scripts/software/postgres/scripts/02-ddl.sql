set timezone to 'Asia/Chongqing';
--
create user fusiondata with nosuperuser replication encrypted password '2024.Fusiondata';
create database fusiondata owner = fusiondata template = template1;

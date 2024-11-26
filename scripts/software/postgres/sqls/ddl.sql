set
  timezone to 'Asia/Chongqing';

create user ultimate
with
  nosuperuser encrypted password '2024.Ultimate';

create database ultimate owner = ultimate template = template1;

create user fusiondata
with
  nosuperuser replication encrypted password '2024.Fusiondata';

create database fusiondata owner = fusiondata template = template1;

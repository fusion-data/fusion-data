set
  timezone to 'Asia/Chongqing';

-- ultimate --
create user ultimate
with
  superuser encrypted password '2024.Ultimate';

alter user ultimate
set
  timezone = 'Asia/Chongqing';

create database ultimate owner = ultimate template = template1;

alter database ultimate
set
  timezone = 'Asia/Chongqing';

-- fusiondata --
create user fusiondata
with
  nosuperuser encrypted password '2025.Fusiondata';

create database fusiondata owner = fusiondata template = template1;

alter database fusiondata
set
  timezone = 'Asia/Chongqing';

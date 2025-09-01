set
  timezone to 'Asia/Chongqing';

-- fusion --
create user fusion
with
  superuser encrypted password '2024.Fusion';

alter user fusion
set
  timezone = 'Asia/Chongqing';

create database fusion owner = fusion template = template1;

alter database fusion
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

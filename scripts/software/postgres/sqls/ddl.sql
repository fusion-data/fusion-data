set
  timezone to 'Asia/Chongqing';

-- fusiondata --
create user fusiondata
with
  superuser encrypted password '2025.Fusiondata';

alter user fusiondata
set
  timezone = 'Asia/Chongqing';

create database fusiondata owner = fusiondata template = template1;

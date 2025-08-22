set
  timezone to 'Asia/Chongqing';

insert into
  user_entity (id, email, "name", "password", status, mfa_enabled, created_at, created_by)
values
  (
    1,
    'admin@hetumind.com',
    'Admin',
    '#1#$argon2id$v=19$m=19456,t=2,p=1$CnoakTKSq9TZxn7TD/1Hmw$Yhff4M7dEMFl3zqNoPGnuOb1uRAcdsrqwXybdZZkeYg',
    100,
    false,
    current_timestamp,
    1
  );

alter sequence user_entity_id_seq restart
with
  2;

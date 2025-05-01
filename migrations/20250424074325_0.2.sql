create table contact
(
    id   integer primary key autoincrement,
    name text
);

create index idx_contact_name on contact (name);

create table sms
(
    id         integer primary key autoincrement,
    timestamp  timestamp not null,
    message    text      not null,
    device     text      not null,
    contact_id integer   not null,
    send       boolean   not null default 0,
    read       boolean   not null default 0
);

create index idx_sms_contact_timestamp on sms (contact_id, timestamp desc);
create index idx_sms_contact_id on sms (contact_id);

create view v_contact as
select c.id, c.name, s.timestamp, s.message, s.read
from contact c
         left join (select *
                    from (select s.*,
                                 row_number() over (partition by contact_id order by timestamp desc) as rn
                          from sms s) sub
                    where rn = 1) s on c.id = s.contact_id;










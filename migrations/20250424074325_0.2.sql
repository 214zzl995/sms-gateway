create table contacts
(
    id   text primary key,
    name text
);

create index idx_contacts_name on contacts (name);
create unique index idx_contacts_name_unique on contacts (name);

create table sms
(
    id         integer primary key autoincrement,
    timestamp  timestamp not null,
    message    text      not null,
    device     text      not null,
    contact_id text      not null,
    send       boolean   not null default 0,
    status     integer   not null default 0
);

create index idx_sms_contact_timestamp on sms (contact_id, timestamp desc);
create index idx_sms_contact_id on sms (contact_id);

create view v_contacts as
select c.id, c.name, s.timestamp, s.message, s.status, s.device
from contacts c
         inner join (select *
                    from (select s.*,
                                 row_number() over (partition by contact_id order by timestamp desc) as rn
                          from sms s) sub
                    where rn = 1) s on c.id = s.contact_id;










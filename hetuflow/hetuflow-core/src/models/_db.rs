use modelsql::postgres::PgRowType;

use super::*;

impl PgRowType for SchedServer {}
impl PgRowType for SchedAgent {}
impl PgRowType for SchedJob {}
impl PgRowType for SchedSchedule {}
impl PgRowType for SchedTask {}
impl PgRowType for SchedTaskInstance {}

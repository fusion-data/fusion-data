use modelsql::postgres::PgRowType;

use super::*;

impl PgRowType for AgentEntity {}
impl PgRowType for JobEntity {}
impl PgRowType for ScheduleEntity {}
impl PgRowType for ServerEntity {}
impl PgRowType for TaskInstanceEntity {}
impl PgRowType for TaskEntity {}

use crate::policy::{PolicyAuditStore, PolicyAuthorization, PolicyGate, PolicyRequest};

use super::BoardService;

impl<AuditStore> BoardService<AuditStore>
where
    AuditStore: PolicyAuditStore,
{
    /// Evaluates and durably audits an execution-start decision before a worker is launched.
    pub(crate) fn authorize_execution_start(
        &mut self,
        policy_gate: &PolicyGate,
        request: PolicyRequest,
    ) -> Result<PolicyAuthorization, AuditStore::Error> {
        policy_gate.authorize_and_record(request, &mut self.repository)
    }
}

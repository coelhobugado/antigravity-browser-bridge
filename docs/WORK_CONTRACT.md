# Contrato de trabalho Antigravity v1

O contrato central está implementado em `cli/src/antigravity/work_service.rs`. Todas as respostas de trabalho carregam `schemaVersion: 1` e usam IDs distintos: `WorkId`, `StepId`, `AttemptId`, `RequestId` e `IdempotencyKey`.

## Estados

```text
created → planning → waiting_for_tab → observing
observing → waiting_for_approval → executing → verifying → completed
executing/verifying → recovering → observing ou executing
qualquer estado não terminal → failed ou cancelled
```

Transições inválidas são rejeitadas antes de alterar o journal. Estados `completed`, `failed` e `cancelled` são terminais.

## Operações MCP

`agent_browser_work_session_start` cria a identidade, consulta abas autorizadas e aplica idempotência. `work_observe`, `work_execute`, `work_verify`, `work_status`, `work_cancel`, `work_checkpoint`, `work_resume`, `work_export`, `work_request_approval` e `work_journal` são adaptadores finos para o `WorkService`.

O adapter MCP não acessa TCP, arquivos de estado da bridge ou lógica de negócio. O transporte autenticado é responsabilidade do runtime local.

## Erros

Os códigos estáveis são `invalid_request`, `invalid_transition`, `transport`, `authorization`, `target`, `navigation`, `policy`, `verification`, `site`, `deadline_exceeded`, `cancelled`, `conflict`, `persistence`, `not_found` e `unsupported`.

## Persistência e recuperação

Cada transição é gravada em `work-journal.jsonl` antes do próximo efeito. Checkpoints incluem estado, plano, aba, geração do documento, efeitos confirmados e resultado. Retomada rejeita schemas incompatíveis com orientação explícita, e chaves de idempotência evitam repetir efeitos confirmados.

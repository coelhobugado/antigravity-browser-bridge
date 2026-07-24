# CI ownership e justificativas

Este arquivo registra por que cada workflow continua no fork e qual área deve mantê-lo.

| Job | Escopo | Motivo | Responsável |
| --- | --- | --- | --- |
| version-sync | versões Node, Rust e dashboard | evita releases inconsistentes | mantenedor do release |
| rust | fmt, clippy e testes determinísticos | feedback rápido sem depender de Chrome | mantenedor Rust |
| rust-cross | compilação e testes por plataforma | detecta regressões de Windows e macOS | mantenedor Rust |
| native-e2e | daemon real e Chrome | valida integração de navegador | mantenedor bridge |
| native-parity | ações documentadas com Chrome | cobre o contrato MCP/CLI em navegador real | mantenedor bridge |
| extension-validation | manifest, ID, scripts e ZIP | impede instalação quebrada | mantenedor extensão |
| baseline | latência, RSS, binários e disco de build | torna regressões de desempenho e crescimento observáveis | mantenedor de release |
| dashboard, sandbox-package, eve-package | pacotes herdados ainda distribuídos | preserva compatibilidade enquanto a remoção é avaliada | mantenedor de pacotes |
| windows-integration | host nativo e ciclo de vida no Windows | reproduz o ambiente principal do usuário | mantenedor Windows |

Jobs herdados da base Vercel devem ser removidos quando o pacote correspondente deixar de ser distribuído. Até lá, são mantidos com justificativa explícita para não criar regressões silenciosas.

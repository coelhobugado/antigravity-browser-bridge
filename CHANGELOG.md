# Changelog

Este arquivo registra somente as mudanças específicas do **Antigravity Browser Bridge**.

Para entender a relação com o projeto-base, consulte [UPSTREAM.md](UPSTREAM.md). O histórico completo do `agent-browser` continua disponível no repositório oficial da Vercel.

## 0.1.0-beta.3

### Correções

- Corrigido o protocolo da extensão e do native host para usar a mesma versão, restaurando o handshake após a conexão.
- Corrigida a instalação para configurar automaticamente `~/.gemini/config/mcp_config.json`, preservar servidores existentes e recusar JSON inválido.
- Removidos módulos experimentais desconectados que simulavam IPC, políticas e adapters sem executar o fluxo real.
- Corrigidos o popup e o content script com nonce criptográfico, renderização segura, autorização persistida por sessão e melhor resolução de elementos.
- Impedido o travamento do `doctor` no Windows ao consultar a versão do Chrome.
- Corrigido o teste de gravação para respeitar a dependência opcional do FFmpeg.
- Corrigido o teste de instalação global para instalar exatamente o arquivo gerado por `npm pack`.

### Melhorias

- Atualizada a extensão para `1.2.0`, com popup para autorizar e revogar abas, status da conexão e atividade recente.
- Atualizados os guias em português e inglês, a documentação da CLI e o skill principal.
- Expandido o CI com validação da extensão, encoding, baselines, empacotamento, SBOM, Clippy, testes Rust e instalação global.

### Contributors

- @coelhobugado
- Antigravity Agent

## 0.1.0-beta.2

### WorkService e estabilidade

- Implementado o `WorkService` tipado para o perfil MCP `antigravity-work`.
- Adicionada máquina de estados, deadlines, idempotência, cancelamento cooperativo, journal, checkpoints, retomada, exportação redigida e status real.
- Separado o adapter MCP da lógica de negócio.
- Adicionado runtime assíncrono sob demanda com lock por instância.
- Atualizados contratos, documentação, validações da extensão, avaliações e artefatos de release.
- Atualizada a extensão para `1.1.2`.

### Contributors

- @coelhobugado

## 0.1.0-beta.1

### Beta inicial

- Adicionado o perfil MCP `antigravity-work` para operar abas explicitamente autorizadas no Chrome já aberto pelo usuário.
- Adicionada extensão genérica para Chrome com ID estável, autorização por aba, observação do DOM, referências de elementos e native messaging.
- Adicionado instalador do native host para Windows, diagnóstico, validação de permissões e ponte autenticada local.
- Adicionados guias de instalação e solução de problemas em português e inglês.
- Removidos scaffolds abandonados do Antigravity e o fluxo de release npm herdado que não fazia parte do produto.

### Créditos

- Construído sobre [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser).

### Contributors

- @coelhobugado

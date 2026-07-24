# Relação com o projeto upstream

O **Antigravity Browser Bridge** é um projeto open source independente e derivado do [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser), distribuído sob a licença Apache 2.0.

Esta página explica com clareza o que foi herdado, o que foi criado especificamente para o Bridge e como créditos e histórico são tratados.

## O que vem do agent-browser

A base upstream fornece partes importantes da infraestrutura técnica, incluindo:

- motor de automação em Rust;
- estrutura principal da CLI;
- operações gerais de navegador;
- parte da infraestrutura MCP;
- componentes de sessão, DOM, seletores e comunicação com Chrome;
- testes, pacotes e ferramentas compartilhadas que ainda permanecem no repositório;
- histórico Git anterior ao início do Antigravity Browser Bridge.

Os autores desses commits continuam aparecendo no histórico e na lista de contribuidores porque o histórico upstream foi preservado.

## O que é específico do Antigravity Browser Bridge

O Bridge adiciona e mantém uma camada própria orientada ao uso do Chrome cotidiano e já autenticado do usuário:

- perfil MCP `antigravity-work`;
- `WorkService` tipado e separado do adapter MCP;
- máquina de estados para tarefas;
- deadlines, idempotência e cancelamento cooperativo;
- journal append-only;
- checkpoints, retomada e recuperação;
- aprovação explícita antes de ações sensíveis;
- verificação posterior no DOM;
- exportação redigida;
- extensão própria para Chrome;
- popup de autorização e revogação de abas;
- native messaging no Windows;
- autorização explícita por aba;
- revogação automática ao mudar de domínio;
- reutilização do Chrome já aberto e de sessões autenticadas existentes;
- instalação e diagnóstico do native host;
- configuração automática do MCP no Antigravity;
- baselines, validações, artefatos e roadmap próprios.

## Identidade do projeto

A descrição recomendada é:

> Antigravity Browser Bridge is an independent open-source project built on top of Vercel's agent-browser. It reuses the upstream Rust automation engine and selected CLI/MCP foundations while adding a dedicated Chrome extension, Windows Native Messaging bridge, explicit per-tab authorization, authenticated-session reuse, and a persistent user-controlled work runtime for browser agents.

O projeto não reivindica autoria sobre o motor upstream. A identidade própria vem das integrações, arquitetura, modelo de segurança, runtime de tarefas, manutenção e direção técnica adicionados pelo Bridge.

## Versionamento

| Componente | Linha de versão |
|---|---:|
| Antigravity Browser Bridge | `0.1.0-beta.x` |
| Extensão Chrome | `1.x` |
| agent-browser upstream | `0.33.x` no ponto atual da base |

As versões são independentes e não devem ser comparadas como se representassem o mesmo produto.

## Changelog

O arquivo [CHANGELOG.md](CHANGELOG.md) registra somente as releases específicas do Antigravity Browser Bridge.

O histórico de releases do projeto-base deve ser consultado no repositório oficial do `agent-browser`. O histórico Git upstream permanece preservado neste repositório por transparência e atribuição.

## Licença e atribuição

O Bridge preserva a licença Apache 2.0 e os avisos aplicáveis. Consulte [LICENSE](LICENSE).

Ao reutilizar ou redistribuir o projeto, preserve os créditos e avisos exigidos pela licença. Novas contribuições ao Bridge também são aceitas sob a licença indicada no repositório.

## Independência

Antigravity Browser Bridge não é um produto oficial da Vercel, Google, Anthropic, X ou LinkedIn.

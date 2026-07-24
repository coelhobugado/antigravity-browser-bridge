# Antigravity Browser Bridge

> Ponte open source entre agentes compatíveis com MCP e o Chrome já autenticado do usuário, com autorização explícita por aba, execução persistente de tarefas e verificação de ações no DOM.

[English documentation](README.en.md) · [Contribuir](CONTRIBUTING.md) · [Segurança](SECURITY.md) · [Roadmap](docs/ANTIGRAVITY_APEX_PLAN.md) · [Changelog](CHANGELOG.md) · [Relação com o upstream](UPSTREAM.md)

> **Status:** `0.1.0-beta.3`. Use com atenção em contas pessoais e sempre revise ações importantes.

![Ícone do Antigravity Browser Bridge](extension/icons/icon-128.png)

## Visão geral das versões

| Componente | Versão atual |
|---|---:|
| Antigravity Browser Bridge | `0.1.0-beta.3` |
| Extensão Chrome | `1.2.0` |
| Base agent-browser | linha upstream `0.33.x` |

As versões são independentes: a extensão, o Bridge e a base upstream não seguem necessariamente a mesma numeração.

## O problema que este projeto resolve

Ferramentas de automação por teclado, coordenadas de tela, PowerShell ou VBS dependem de foco, tempo de carregamento e estado visual. Elas podem clicar na janela errada, perder mudanças da interface ou relatar sucesso sem comprovação.

O Antigravity Browser Bridge oferece uma camada mais determinística para agentes:

1. o cliente MCP inicia o servidor;
2. o servidor conversa com um native host local;
3. o native host mantém uma ponte autenticada com a extensão;
4. o usuário autoriza explicitamente cada aba;
5. a extensão observa o DOM e gera referências estruturadas;
6. o `WorkService` controla estado, aprovação, execução, verificação e recuperação;
7. o agente registra o resultado e pode retomar tarefas interrompidas sem repetir efeitos confirmados.

```text
Agente MCP
   ↓
Servidor MCP / CLI Rust
   ↓
WorkService tipado
   ↓
Native Messaging Host
   ↓
Extensão Chrome
   ↓
Aba autorizada e DOM
```

## O que diferencia este projeto

O projeto é derivado do [agent-browser da Vercel](https://github.com/vercel-labs/agent-browser), sob licença Apache 2.0, mas adiciona uma camada própria voltada ao uso seguro do navegador cotidiano do usuário:

- perfil MCP `antigravity-work`;
- reutilização do Chrome já aberto e de sessões autenticadas existentes;
- extensão própria para Chrome;
- native messaging no Windows;
- autorização explícita por aba;
- persistência apenas dentro do mesmo domínio;
- revogação automática ao mudar de domínio;
- `WorkService` tipado com máquina de estados;
- deadlines, idempotência e cancelamento cooperativo;
- journal append-only, checkpoints e retomada;
- aprovação explícita antes de ações sensíveis;
- verificação posterior no DOM;
- exportação redigida para reduzir exposição de dados sensíveis;
- diagnóstico e instalação automática do native host e do MCP;
- roadmap próprio para recuperação de falhas, evidência de sucesso e controle de risco.

Essas adições transformam a base upstream em uma ponte específica para agentes que precisam trabalhar com sessões reais sem receber acesso silencioso a todo o navegador.

Consulte [UPSTREAM.md](UPSTREAM.md) para ver o que é herdado, o que é específico do Bridge e como o histórico upstream é preservado.

## Runtime de tarefas

O perfil `antigravity-work` usa um `WorkService` separado do adapter MCP. Cada tarefa possui identidade própria, estado, tentativas, deadline, chave de idempotência e journal persistente.

Fluxo principal:

```text
created → planning → waiting_for_tab → observing
observing → waiting_for_approval → executing → verifying → completed
executing/verifying → recovering → observing ou executing
qualquer estado não terminal → failed ou cancelled
```

Operações disponíveis incluem:

- iniciar sessão;
- observar a aba autorizada;
- solicitar aprovação;
- executar uma etapa;
- verificar o resultado;
- consultar status e journal;
- cancelar;
- criar checkpoint;
- retomar;
- exportar estado redigido.

Transições inválidas são rejeitadas antes de alterar o journal. Efeitos confirmados são protegidos contra repetição por idempotência. O contrato completo está em [docs/WORK_CONTRACT.md](docs/WORK_CONTRACT.md).

## Casos de uso

- preparar uma publicação no LinkedIn e pedir confirmação antes de publicar;
- preencher formulários em abas já autenticadas;
- consultar sistemas web sem compartilhar cookies com o modelo;
- automatizar tarefas repetitivas com verificação posterior no DOM;
- retomar fluxos interrompidos a partir de checkpoints;
- integrar Antigravity ou outro cliente MCP a fluxos no navegador;
- testar agentes em aplicações web reais com limites explícitos de autorização.

## Estado atual

Funciona atualmente:

- conexão MCP pelo perfil `antigravity-work`;
- uso do Chrome já aberto;
- listagem de abas autorizadas;
- observação do DOM com referências;
- clique, foco, preenchimento, digitação e leitura de texto;
- persistência da autorização no mesmo domínio;
- revogação automática em navegação entre domínios;
- ID estável da extensão;
- popup para autorizar e revogar abas;
- status da conexão e atividade recente;
- instalação e diagnóstico do native host no Windows;
- configuração automática do MCP no Antigravity;
- WorkService com journal, checkpoints, retomada, idempotência e verificação.

Limitações da versão beta:

- menus dinâmicos, diálogos e editores complexos ainda precisam de operações adicionais;
- interfaces de sites podem mudar sem aviso;
- o instalador nativo está concentrado no Windows;
- nem todas as ferramentas planejadas foram implementadas;
- uma resposta `not_implemented` nunca deve ser tratada como sucesso.

## Requisitos

- Windows 10 ou 11;
- Google Chrome Stable, Beta, Dev ou Canary;
- Node.js 24+;
- pnpm 11+;
- Rust estável;
- Antigravity ou outro cliente MCP compatível.

## Instalação a partir do código

```powershell
git clone https://github.com/coelhobugado/antigravity-browser-bridge.git
cd antigravity-browser-bridge
pnpm install
pnpm build:native
```

O executável será gerado em `bin/agent-browser-win32-x64.exe`.

Registre e valide o native host:

```powershell
.\bin\agent-browser-win32-x64.exe antigravity install
.\bin\agent-browser-win32-x64.exe antigravity doctor
.\bin\agent-browser-win32-x64.exe antigravity permissions
```

## Instalação da extensão

Baixe a extensão na [página de Releases](https://github.com/coelhobugado/antigravity-browser-bridge/releases) ou use a pasta `extension`.

1. Extraia o ZIP para uma pasta permanente.
2. Abra `chrome://extensions`.
3. Ative o modo do desenvolvedor.
4. Clique em **Carregar sem compactação**.
5. Selecione a pasta que contém `manifest.json`.
6. Confirme que a extensão `1.2.0` foi carregada corretamente.

O ID oficial é estável e já é autorizado pelo instalador.

## Configuração MCP no Antigravity

O comando `antigravity install` registra o native host e configura automaticamente o servidor MCP no arquivo:

```text
C:\Users\SEU_USUARIO\.gemini\config\mcp_config.json
```

O instalador preserva os servidores existentes e se recusa a sobrescrever um arquivo JSON inválido. A configuração resultante contém:

```json
{
  "mcpServers": {
    "agent-browser": {
      "command": "C:\\CAMINHO\\antigravity-browser-bridge\\bin\\agent-browser-win32-x64.exe",
      "args": ["mcp", "--tools", "antigravity-work"]
    }
  }
}
```

Não é necessário informar o ID da extensão. Depois da instalação, recarregue o Antigravity e verifique se as ferramentas `agent_browser_work_*` aparecem.

## Uso seguro

1. Abra o site no Chrome.
2. Abra o popup da extensão e autorize somente a aba necessária.
3. Peça ao agente para observar antes de agir.
4. Exija confirmação antes de publicar, excluir, comprar ou enviar mensagens.
5. Peça uma nova observação após a ação para verificar o resultado.

Exemplo:

```text
Use as ferramentas agent_browser_work. Liste as abas autorizadas, observe a aba do LinkedIn, prepare a publicação e peça minha confirmação antes de clicar em Publicar. Depois, observe novamente e confirme o resultado.
```

A extensão não recebe acesso silencioso a todo o navegador. Não exponha cookies, tokens, perfis do navegador ou arquivos locais de estado.

## Desenvolvimento e testes

```powershell
pnpm install
pnpm version:sync
cargo fmt --manifest-path cli\Cargo.toml -- --check
cargo test --manifest-path cli\Cargo.toml
node --check extension\background.js
node --check extension\content.js
```

Testes end-to-end com Chrome:

```powershell
cargo test e2e --manifest-path cli\Cargo.toml -- --ignored --test-threads=1
```

O CI também valida sincronização de versões, Clippy, extensão, encoding, baselines, empacotamento, SBOM e instalação global do pacote.

## Como contribuir

Contribuições são bem-vindas, especialmente em:

- testes em aplicações reais;
- suporte a menus, diálogos e editores;
- compatibilidade com outros clientes MCP;
- instaladores para outros sistemas;
- segurança e recuperação de falhas;
- documentação em português e inglês.

Leia [CONTRIBUTING.md](CONTRIBUTING.md) antes de abrir um pull request. Vulnerabilidades devem ser relatadas conforme [SECURITY.md](SECURITY.md).

## Roadmap

As prioridades atuais são:

- confirmação de ações por nível de risco;
- operações mais ricas para interfaces dinâmicas;
- recuperação após mudanças do DOM;
- evidência automática de sucesso ou falha;
- instaladores e pacotes assinados;
- testes em X, LinkedIn e aplicações genéricas;
- suporte mais amplo a clientes MCP e sistemas operacionais.

O plano detalhado está em [docs/ANTIGRAVITY_APEX_PLAN.md](docs/ANTIGRAVITY_APEX_PLAN.md).

## Créditos e licença

Baseado no [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser). O projeto preserva a licença Apache 2.0 e os avisos aplicáveis. Consulte [LICENSE](LICENSE) e [UPSTREAM.md](UPSTREAM.md).

Antigravity Browser Bridge não é um produto oficial da Vercel, Google, X, LinkedIn ou Anthropic.

# Antigravity Browser Bridge

> Ponte open source entre agentes compatíveis com MCP e o Chrome já autenticado do usuário, com autorização explícita por aba e verificação de ações no DOM.

[English documentation](README.en.md) · [Contribuir](CONTRIBUTING.md) · [Segurança](SECURITY.md) · [Roadmap](docs/ANTIGRAVITY_APEX_PLAN.md) · [Changelog](CHANGELOG.md)

> **Status:** `0.1.0-beta.1`. Use com atenção em contas pessoais e sempre revise ações importantes.

![Ícone do Antigravity Browser Bridge](extension/icons/icon-128.png)

## O problema que este projeto resolve

Ferramentas de automação por teclado, coordenadas de tela, PowerShell ou VBS dependem de foco, tempo de carregamento e estado visual. Elas podem clicar na janela errada, perder mudanças da interface ou relatar sucesso sem comprovação.

O Antigravity Browser Bridge oferece uma camada mais determinística para agentes:

1. o cliente MCP inicia o servidor;
2. o servidor conversa com um native host local;
3. o native host mantém uma ponte autenticada com a extensão;
4. o usuário autoriza explicitamente cada aba;
5. a extensão observa o DOM e gera referências estruturadas;
6. o agente lê, preenche, clica e verifica o resultado por essas referências.

```text
Agente MCP
   ↓
Servidor MCP / CLI Rust
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
- extensão genérica para Chrome;
- native messaging no Windows;
- autorização explícita por aba;
- persistência apenas dentro do mesmo domínio;
- revogação automática ao mudar de domínio;
- diagnóstico e instalação do native host;
- orientação para confirmação de ações públicas, destrutivas ou financeiras;
- roadmap próprio para recuperação de falhas, evidência de sucesso e controle de risco.

Essas adições transformam a base upstream em uma ponte específica para agentes que precisam trabalhar com sessões reais sem receber acesso silencioso a todo o navegador.

## Casos de uso

- preparar uma publicação no LinkedIn e pedir confirmação antes de publicar;
- preencher formulários em abas já autenticadas;
- consultar sistemas web sem compartilhar cookies com o modelo;
- automatizar tarefas repetitivas com verificação posterior no DOM;
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
- instalação e diagnóstico do native host no Windows.

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

Baixe `antigravity-browser-bridge-extension-v0.1.0-beta.1.zip` na página de Releases ou use a pasta `extension`.

1. Extraia o ZIP para uma pasta permanente.
2. Abra `chrome://extensions`.
3. Ative o modo do desenvolvedor.
4. Clique em **Carregar sem compactação**.
5. Selecione a pasta que contém `manifest.json`.
6. Confirme que a extensão foi carregada corretamente.

O ID oficial é estável e já é autorizado pelo instalador.

## Configuração MCP no Antigravity

Edite o arquivo:

```text
C:\Users\SEU_USUARIO\.gemini\config\mcp_config.json
```

Adicione a entrada sem apagar servidores já existentes:

```json
{
  "mcpServers": {
    "antigravity-browser-bridge": {
      "command": "C:\\CAMINHO\\antigravity-browser-bridge\\bin\\agent-browser-win32-x64.exe",
      "args": ["mcp", "--tools", "antigravity-work"]
    }
  }
}
```

Depois, recarregue o Antigravity e verifique se as ferramentas `agent_browser_work_*` aparecem.

## Uso seguro

1. Abra o site no Chrome.
2. Autorize somente a aba necessária.
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

Baseado no [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser). O projeto preserva a licença Apache 2.0 e os avisos aplicáveis. Consulte [LICENSE](LICENSE).

Antigravity Browser Bridge não é um produto oficial da Vercel, Google, X, LinkedIn ou Anthropic.
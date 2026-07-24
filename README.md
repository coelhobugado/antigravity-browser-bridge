# Antigravity Browser Bridge

> Projeto beta. Use com atenção em contas pessoais e sempre revise ações importantes.

[English documentation](README.en.md)

O Antigravity Browser Bridge conecta agentes compatíveis com MCP ao Chrome que você já usa. A extensão trabalha com abas autorizadas pelo usuário, preservando sessões já autenticadas em serviços como X, LinkedIn e outras aplicações web.

O projeto foi construído sobre o [agent-browser da Vercel](https://github.com/vercel-labs/agent-browser), licenciado sob Apache 2.0. O motor Rust, a automação via navegador e partes da interface CLI vêm dessa base. Esta versão adiciona a integração MCP `antigravity-work`, uma extensão genérica para Chrome, native messaging no Windows e um modelo explícito de autorização por aba.

![Ícone do Antigravity Browser Bridge](extension/icons/icon-128.png)

## Por que ele existe

Automação por teclado, PowerShell, VBS ou coordenadas de tela depende de foco, tempo de carregamento e permissões do Windows. Essa abordagem pode digitar na janela errada ou concluir incorretamente que uma ação funcionou.

O Antigravity Browser Bridge usa outra arquitetura:

1. O Antigravity inicia o servidor MCP do `agent-browser`.
2. O servidor conversa com o native host instalado no Windows.
3. O native host mantém uma ponte autenticada com a extensão.
4. A extensão observa o DOM da aba autorizada e produz referências estáveis.
5. O agente preenche, clica e lê elementos por essas referências.

Isso torna a operação mais determinística, mas não transforma qualquer modelo em um agente perfeito. O modelo ainda precisa observar a página, confirmar o resultado e tratar mudanças de interface.

## Estado atual

Esta versão é `0.1.0-beta.1`.

Funciona atualmente:

- conexão MCP com o perfil `antigravity-work`;
- uso do Chrome já aberto e das sessões autenticadas existentes;
- autorização explícita por aba;
- listagem das abas autorizadas;
- observação do DOM com referências;
- clique, foco, preenchimento, digitação e leitura de texto;
- persistência da autorização durante navegação no mesmo domínio;
- revogação automática ao mudar de domínio;
- ID fixo da extensão, sem copiar o ID para a IA;
- instalação e diagnóstico do native host no Windows.

Ainda é beta:

- alguns fluxos complexos, menus dinâmicos e diálogos exigem novas operações;
- interfaces de sites podem mudar sem aviso;
- ações destrutivas ou públicas devem exigir confirmação;
- suporte do instalador nativo está concentrado no Windows;
- nem todas as ferramentas planejadas em `antigravity-work` foram implementadas.

Uma resposta `not_implemented` nunca deve ser interpretada como sucesso.

## Requisitos

- Windows 10 ou 11;
- Google Chrome, incluindo Stable, Beta, Dev ou Canary;
- Node.js 24 ou superior;
- pnpm 11 ou superior;
- Rust estável;
- Antigravity ou outro cliente MCP compatível.

## Instalação a partir do código

```powershell
git clone https://github.com/coelhobugado/antigravity-browser-bridge.git
cd antigravity-browser-bridge
pnpm install
pnpm build:native
```

O comando gera o executável compatível com o Windows em `bin/agent-browser-win32-x64.exe`.

Registre o native host:

```powershell
.\bin\agent-browser-win32-x64.exe antigravity install
.\bin\agent-browser-win32-x64.exe antigravity doctor
.\bin\agent-browser-win32-x64.exe antigravity permissions
```

## Instalação da extensão

Você pode baixar `antigravity-browser-bridge-extension-v0.1.0-beta.1.zip` na página de Releases ou usar diretamente a pasta `extension` deste repositório.

1. Extraia o ZIP para uma pasta permanente.
2. Abra `chrome://extensions`.
3. Ative o modo do desenvolvedor.
4. Clique em **Carregar sem compactação**.
5. Selecione a pasta extraída que contém `manifest.json`.
6. Confirme que a extensão mostra a versão `1.1.1`.

O ID oficial é estável, mas você não precisa copiá-lo nem passá-lo para a IA. O instalador já autoriza esse ID.

## Configuração do MCP no Antigravity

O Antigravity precisa conhecer o servidor MCP. No Windows, edite:

```text
C:\Users\SEU_USUARIO\.gemini\config\mcp_config.json
```

Use um caminho absoluto para o executável:

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

Se o arquivo já tiver outros servidores, adicione somente a entrada `antigravity-browser-bridge` dentro de `mcpServers`. Não apague as configurações existentes.

Uma IA com acesso ao sistema de arquivos pode fazer essa configuração, mas deve:

- localizar o executável real;
- preservar os servidores MCP existentes;
- usar JSON válido;
- reiniciar ou recarregar o Antigravity após a alteração;
- testar se as ferramentas `agent_browser_work_*` aparecem.

## Uso

1. Abra o site desejado no Chrome.
2. Clique no ícone da extensão nessa aba.
3. Confirme o indicador `ON`.
4. Peça ao agente para observar a aba antes de agir.
5. Para publicações, exclusões e mensagens, peça que ele confirme o resultado no DOM.

Exemplo de instrução:

```text
Use as ferramentas agent_browser_work. Liste as abas autorizadas, observe a aba do LinkedIn, prepare a publicação e peça minha confirmação antes de clicar em Publicar. Depois, observe novamente e confirme o resultado.
```

## Segurança

A extensão não recebe acesso silencioso a todo o navegador. Cada aba precisa ser autorizada pelo clique do usuário. A autorização é removida ao navegar para outro domínio ou fechar a aba.

Boas práticas:

- autorize apenas as abas necessárias;
- exija confirmação antes de publicar, excluir, comprar ou enviar mensagens;
- não exponha tokens, cookies ou o arquivo de estado da ponte;
- execute somente builds e releases confiáveis;
- revise logs e respostas estruturadas em caso de dúvida.

## Diagnóstico

```powershell
.\bin\agent-browser-win32-x64.exe antigravity doctor
.\bin\agent-browser-win32-x64.exe antigravity permissions
```

Se aparecer `Access to the specified native messaging host is forbidden`:

1. verifique se a extensão é a versão incluída nesta release;
2. execute novamente `antigravity install`;
3. recarregue a extensão em `chrome://extensions`;
4. feche completamente o Chrome e abra novamente.

Se aparecer `Receiving end does not exist`, recarregue a aba e clique novamente no ícone. Páginas internas como `chrome://extensions` não aceitam injeção de content script.

## Desenvolvimento e testes

```powershell
pnpm install
pnpm version:sync
cargo fmt --manifest-path cli\Cargo.toml -- --check
cargo test --manifest-path cli\Cargo.toml
node --check extension\background.js
node --check extension\content.js
```

Os testes de ponta a ponta precisam do Chrome:

```powershell
cargo test e2e --manifest-path cli\Cargo.toml -- --ignored --test-threads=1
```

## Roadmap

O plano técnico detalhado está em [docs/ANTIGRAVITY_APEX_PLAN.md](docs/ANTIGRAVITY_APEX_PLAN.md).

As prioridades são:

- confirmação de ações por nível de risco;
- operações mais ricas para menus, diálogos e editores;
- recuperação após mudanças do DOM;
- evidência automática de sucesso ou falha;
- instaladores e pacotes assinados;
- cobertura de testes em X, LinkedIn e aplicações genéricas;
- redução adicional do tamanho de distribuição e dos artefatos de desenvolvimento.

## Créditos e licença

Baseado no [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser). Este projeto preserva a licença Apache 2.0 e os avisos aplicáveis. Consulte [LICENSE](LICENSE).

Antigravity Browser Bridge não é um produto oficial da Vercel, Google, X ou LinkedIn.

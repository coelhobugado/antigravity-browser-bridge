# Plano de evolução do Agent Browser para Antigravity

## 1. Resumo executivo

O projeto atual ainda não é um runtime de trabalho comparável ao Codex Work ou Claude Cowork. Ele é uma bifurcação tecnicamente forte do `agent-browser` da Vercel, acrescida de uma camada Antigravity que, no estado atual, é principalmente estrutural: há tipos, nomes de ferramentas, contratos iniciais e arquivos de integração, mas quase todo o comportamento novo ainda é stub, TODO ou código desconectado do executor real.

O ativo mais valioso do projeto não é a extensão recém-criada. É o núcleo herdado e já maduro:

- automação via Chrome DevTools Protocol;
- snapshots de acessibilidade e referências estáveis;
- comandos de navegação e interação;
- múltiplas abas, frames, downloads, diálogos, rede, cookies e storage;
- sessões persistentes e restauração;
- políticas de ação e confirmações;
- MCP tipado com paridade de CLI;
- diffs, tracing, gravação, diagnóstico e recuperação do daemon.

A estratégia correta é transformar esse núcleo em um runtime agentivo único e integrar o Antigravity a ele de maneira nativa. A extensão deve ser uma ponte segura para o navegador real do usuário e suas sessões autenticadas, não um segundo motor de automação baseado em seletores frágeis.

### Diagnóstico em uma frase

Hoje o projeto expõe a aparência de uma camada “Work”, mas não possui ainda o loop confiável de execução, verificação, persistência, aprovação e recuperação que faria essa camada cumprir o que promete.

## 2. Estado atual verificado

### 2.1 Base e tamanho do fork

O repositório parte do `agent-browser` v0.33.0 da Vercel. Desde o upstream, foram acrescentados aproximadamente 975 linhas em 38 arquivos, distribuídas entre:

- `cli/src/antigravity/`;
- `cli/src/work/`;
- dez ferramentas MCP “Work”;
- ADRs e contratos iniciais;
- uma extensão Chrome com Native Messaging.

Esse volume é pequeno diante da ambição do produto. Os módulos novos definem a intenção arquitetural, mas não implementam um runtime completo.

### 2.2 O que realmente funciona

O núcleo original oferece uma superfície ampla e madura de automação. Ele deve continuar sendo a camada canônica de execução do navegador.

Os seguintes elementos novos também têm valor como ponto de partida:

- `ObservationPacket` esboça uma observação multimodal e contextual;
- `WorkAction` prevê idempotência, precondições, pós-condições e risco;
- `Journal` usa SQLite em modo WAL;
- há conceitos iniciais de checkpoint, artefato, recuperação, aprovação e papéis de subagente;
- o perfil MCP `antigravity-work` já está registrado;
- existe uma separação inicial entre runtime, integração e desktop provider.

### 2.3 O que é apenas fachada

As dez ferramentas MCP “Work” retornam mensagens fixas de sucesso. Seus schemas aceitam objetos vazios e não validam goal, sessão, ação, observação ou resultado.

Os módulos abaixo não possuem implementação operacional:

- instalador Antigravity;
- detector e doctor da integração;
- validação de permissões;
- feature flags;
- telemetria local;
- registro de versões;
- sidecar;
- desktop provider;
- secret manager;
- coordenação de agentes;
- estratégias reais de recovery.

Quase todos os tipos em `cli/src/work/` não são usados fora de suas próprias definições. Portanto, não há um pipeline que conecte:

`goal → observação → decisão → política → ação → verificação → journal → checkpoint → resultado`

### 2.4 Problemas concretos da extensão atual

A extensão atual:

- injeta `content.js` em todas as páginas e em todos os frames;
- solicita `<all_urls>`, `tabs`, `scripting`, `storage` e Native Messaging;
- redireciona comandos especificamente para X/Twitter;
- usa `document.querySelector` como mecanismo primário;
- retorna apenas URL, título e quantidade de elementos como observação;
- não possui modelo de consentimento por site, aba, tarefa ou duração;
- não autentica mensagens do processo local;
- não limita tamanho, tipo, origem ou frequência de comandos;
- não implementa timeouts, correlação robusta, cancelamento ou backpressure;
- contém strings com codificação corrompida;
- depende de ID de extensão e caminhos absolutos da máquina do desenvolvedor.

Há ainda uma incompatibilidade fatal no transporte:

- o `native-shim` tenta conectar em `127.0.0.1:4850`;
- `chrome_extension.rs` escuta em `127.0.0.1:4849`;
- o host registrado no Chrome é o `native-shim`;
- não existe listener em 4850 no runtime;
- o código de Native Messaging dentro do executável principal não é alcançado nesse caminho;
- um lado usa mensagens JSON delimitadas por nova linha, enquanto o outro implementa HTTP parcial;
- respostas são sobrescritas em um arquivo absoluto chamado `extension_output.txt`.

Na prática, a extensão e o daemon não formam hoje um canal funcional de ponta a ponta.

### 2.5 Origem dos mais de 5 GB

O repositório ocupa mais de 5 GB porque `cli/target/debug` contém aproximadamente 5,774 GB de artefatos de compilação Rust. A maior parte é composta por:

- caches incrementais;
- arquivos PDB de depuração do Windows;
- bibliotecas `.rlib`;
- múltiplas versões intermediárias do binário e dependências.

Isso não representa o tamanho do produto final e não é causado por dados do LinkedIn, redes sociais ou pelo Chrome. A pasta está corretamente ignorada pelo Git e pode ser apagada com `cargo clean --manifest-path cli/Cargo.toml`. O próximo build a recriará.

O plano deve adicionar:

- comando de limpeza documentado;
- `CARGO_TARGET_DIR` compartilhado e fora do repositório para desenvolvimento;
- perfil `dev` com `debug = 1` ou `debug = 0` quando PDB completo não for necessário;
- `incremental = false` em CI e builds descartáveis;
- rotina de diagnóstico que mostre o tamanho de caches;
- budget de tamanho para binários release e pacotes;
- exclusão do `native-shim/target` no `.gitignore`.

## 3. Limite realista de qualidade

Uma ferramenta de navegador não muda a capacidade fundamental do modelo do Antigravity. Se o modelo falha em decomposição, mantém pouca memória de trabalho ou não verifica resultados, uma API maior pode até piorar seu desempenho.

O runtime pode compensar parte dessa deficiência ao:

- reduzir escolhas por etapa;
- fornecer observações compactas e relevantes;
- impor uma máquina de estados;
- recusar sucesso sem evidência;
- manter memória e checkpoints fora do contexto do modelo;
- executar ações idempotentes;
- recuperar automaticamente de falhas conhecidas;
- exigir aprovação apenas quando o risco justifica;
- oferecer workflows de alto nível para tarefas recorrentes.

O objetivo mensurável não deve ser “ser tão inteligente quanto GPT‑5.6”. Deve ser atingir taxas comparáveis de conclusão em uma suíte definida de tarefas de navegador, com limites explícitos de custo, latência, segurança e intervenção humana.

## 4. Arquitetura alvo

### 4.1 Princípio central

Deve existir um único runtime canônico. CLI, MCP, skill do Antigravity, extensão e futuramente desktop devem ser adaptadores finos sobre esse runtime.

```text
Antigravity UI ou CLI
        │
        ├── Skill e política de uso
        │
        └── MCP compacto e tipado
                  │
           Work Orchestrator
                  │
      ┌───────────┼────────────┐
      │           │            │
  Policy      Journal      Artifact Store
      │           │            │
      └───────────┼────────────┘
                  │
          Browser Runtime único
                  │
        ┌─────────┴─────────┐
        │                   │
 Chrome gerenciado    Chrome do usuário
 via CDP              via conector consentido
```

### 4.2 Work Orchestrator

Implementar uma máquina de estados durável:

```text
Created
  → Planning
  → Observing
  → AwaitingApproval, quando necessário
  → Executing
  → Verifying
  → Recovering, se a verificação falhar
  → Completed, Failed ou Cancelled
```

Cada transição deve:

- possuir `task_id`, `step_id`, `attempt` e `idempotency_key`;
- registrar input, output, duração e versão dos schemas;
- aceitar cancelamento;
- impor deadline;
- persistir antes e depois de efeitos externos;
- nunca marcar uma tarefa como concluída sem uma evidência verificável.

### 4.3 Observação

O `ObservationPacket` deve ser produzido a partir das capacidades já existentes do runtime, não por um novo extrator simplificado na extensão.

Uma observação deve conter:

- URL, título, origem, aba e frame ativos;
- árvore acessível compacta com refs estáveis;
- elementos interativos visíveis;
- foco, seleção, formulários modificados e diálogos;
- screenshot opcional, sob demanda ou por incerteza;
- mudanças desde a observação anterior;
- erros de console e falhas de rede relevantes;
- downloads em progresso;
- estado de carregamento e estabilidade;
- conteúdo marcado por origem e nível de confiança;
- budget e truncamento explícitos.

O modelo deve receber primeiro o diff compacto. Snapshot completo e imagem entram apenas quando necessários.

### 4.4 Ação

Toda ação deve usar os comandos canônicos existentes:

- refs acessíveis como primeira escolha;
- texto, role e label como fallback semântico;
- CSS somente como fallback de último nível;
- coordenadas apenas para canvas ou casos sem DOM utilizável.

O executor deve:

1. validar schema;
2. resolver o alvo;
3. conferir precondições;
4. avaliar risco e política;
5. registrar intenção;
6. executar uma vez;
7. observar o efeito;
8. validar pós-condições;
9. registrar evidência;
10. tentar recovery limitado ou escalar.

### 4.5 Verificação

Separar “ação enviada” de “resultado atingido”.

Exemplos:

- clicar em “Publicar” não prova que o post foi publicado;
- enviar um formulário não prova que o backend aceitou;
- digitar em um editor não prova que o texto persistiu;
- abrir um perfil não prova que é o perfil correto.

Cada workflow deve possuir verificadores por:

- URL;
- texto ou role;
- mudança de DOM;
- resposta de rede;
- aparecimento de artefato;
- estado de download;
- comparação visual;
- regra específica do domínio.

### 4.6 Recovery

Implementar uma taxonomia real:

- referência expirada;
- elemento coberto;
- navegação inesperada;
- aba ou frame incorreto;
- sessão desconectada;
- autenticação expirada;
- rate limit;
- desafio de segurança ou CAPTCHA;
- modal inesperado;
- rede instável;
- ação sem efeito;
- resultado ambíguo.

Cada classe deve ter retries limitados, backoff, nova observação e fallback definidos. CAPTCHA, autenticação sensível e mudanças de identidade devem sempre escalar ao usuário.

## 5. Integração com o Chrome real do usuário

### 5.1 Papel correto da extensão

A extensão deve:

- mostrar estado de conexão e tarefa ativa;
- pedir consentimento granular;
- permitir anexar aba atual, conjunto de abas ou domínio;
- emitir metadados necessários para descobrir e autorizar targets;
- exibir ações pendentes de alto risco;
- permitir pausar, cancelar e revogar acesso;
- manter um canal Native Messaging autenticado;
- evitar implementar automação de DOM duplicada quando CDP estiver disponível.

### 5.2 Modos de conexão

Suportar dois modos claramente separados:

1. **Navegador gerenciado**: perfil isolado controlado pelo runtime. É o modo mais previsível e seguro.
2. **Navegador pessoal anexado**: abas explicitamente autorizadas do Chrome do usuário. É o modo necessário para LinkedIn e redes sociais já autenticadas.

O modo pessoal deve começar com acesso somente à aba autorizada. Expansão para outras abas ou domínios exige consentimento adicional.

### 5.3 Protocolo do conector

Substituir a combinação atual de HTTP parcial, TCP e newline JSON por um protocolo versionado único.

Envelope mínimo:

```json
{
  "protocolVersion": "1.0",
  "messageId": "uuid",
  "correlationId": "uuid",
  "sessionId": "uuid",
  "tabId": 123,
  "origin": "https://www.linkedin.com",
  "type": "command",
  "capability": "tab.attach",
  "deadlineMs": 15000,
  "payload": {}
}
```

Requisitos:

- framing Native Messaging apenas no trecho Chrome ↔ host;
- IPC local autenticado entre host e daemon;
- token efêmero criado pelo daemon;
- ACL do usuário atual;
- limite de tamanho;
- timeouts;
- heartbeat;
- reconexão com replay seguro apenas para mensagens idempotentes;
- correlação de request e response;
- version negotiation;
- erros estruturados;
- nenhuma escrita em arquivo absoluto;
- logs no diretório de dados da aplicação com redaction.

### 5.4 Permissões

Trocar permissões amplas por concessões progressivas sempre que a plataforma permitir:

- `activeTab` para acesso iniciado pelo usuário;
- host permissions opcionais por domínio;
- sem injeção automática em todos os frames de todos os sites;
- lista visível de domínios autorizados;
- expiração por tarefa, sessão ou tempo;
- botão de revogação imediata;
- indicador persistente enquanto o agente controla uma aba.

### 5.5 LinkedIn e redes sociais

Não criar código especial por site dentro do transporte base. Criar “domain adapters” opcionais, versionados e testados:

- LinkedIn;
- X;
- Instagram;
- Facebook;
- outros.

O core continua genérico. Adaptadores podem oferecer operações de alto nível como `linkedin.read_profile` ou `linkedin.prepare_post`, mas devem delegar interação ao runtime canônico.

Ações de publicação, mensagem, conexão, seguir, deletar, comprar ou alterar configurações devem passar pelo policy engine. Preparar conteúdo e publicar conteúdo são capacidades distintas.

## 6. MCP e integração nativa com Antigravity

### 6.1 Problema atual

O perfil `antigravity-work` contém ferramentas com schema vazio e respostas de sucesso falsas. Isso ensina o agente a acreditar que concluiu trabalho que não foi realizado.

### 6.2 Superfície recomendada

Manter uma API compacta orientada ao ciclo de trabalho:

- `work_start(goal, constraints, browser_mode, allowed_origins)`;
- `work_observe(task_id, detail, since_observation_id)`;
- `work_act(task_id, observation_id, action, expected_outcome)`;
- `work_verify(task_id, success_criteria)`;
- `work_checkpoint(task_id, summary)`;
- `work_resume(task_id)`;
- `work_approve(approval_id, decision)`;
- `work_status(task_id)`;
- `work_cancel(task_id)`;
- `work_export(task_id, formats)`.

Ferramentas genéricas de baixo nível podem continuar disponíveis em um perfil avançado. O perfil padrão do Antigravity deve ser pequeno para reduzir seleção errada e custo de contexto.

### 6.3 Contratos

Todos os schemas precisam:

- `additionalProperties: false`;
- campos obrigatórios;
- enums;
- limites de tamanho;
- descrições de risco;
- exemplos válidos;
- annotations MCP corretas para leitura, escrita, idempotência e mundo aberto;
- erros estruturados e acionáveis.

As ferramentas MCP devem chamar a mesma camada de aplicação usada pela CLI. Não pode haver implementação paralela.

### 6.4 Skill do Antigravity

A skill precisa ensinar um workflow obrigatório:

1. iniciar tarefa com goal e critérios de sucesso;
2. observar antes de agir;
3. referenciar o `observation_id`;
4. realizar uma ação pequena por vez;
5. verificar efeitos;
6. usar diff antes de pedir snapshot completo;
7. solicitar aprovação quando indicado;
8. checkpoint em tarefas longas;
9. nunca declarar sucesso sem evidência.

O instalador deve detectar os locais reais de configuração do Antigravity e Antigravity CLI, instalar de forma idempotente, validar a instalação e conseguir desfazer apenas os artefatos que ele próprio criou.

## 7. Segurança e privacidade

Dar acesso a LinkedIn, redes sociais e todas as abas transforma o runtime em software de alta sensibilidade. Segurança deve ser parte da arquitetura, não uma etapa final.

### 7.1 Threat model obrigatório

Cobrir:

- prompt injection em páginas;
- exfiltração de cookies, tokens e mensagens;
- processo local malicioso conectando à porta do daemon;
- extensão ou ID adulterado;
- confusão entre abas, perfis e identidades;
- ações destrutivas sem confirmação;
- replay de comandos;
- vazamento em logs, screenshots, traces, HAR e journal;
- dependências comprometidas;
- atualização maliciosa;
- sites tentando induzir o agente a expandir permissões.

### 7.2 Separação de conteúdo

Toda observação deve preservar proveniência:

- instrução do usuário;
- política do sistema;
- conteúdo da web não confiável;
- conteúdo produzido pelo agente;
- segredo ou dado sensível.

Texto da página nunca pode ser tratado como instrução de sistema. Ações pedidas pela página que expandam acesso, revelem segredos ou mudem o objetivo devem ser recusadas ou escaladas.

### 7.3 Policy engine

Classificar ações por efeito, não apenas por comando:

- leitura local;
- navegação;
- edição reversível;
- comunicação externa;
- publicação;
- alteração de conta;
- transação financeira;
- exclusão;
- download ou upload;
- exposição de segredo.

A política deve considerar domínio, identidade, audiência, quantidade, horário, tarefa e histórico. Uma aprovação deve estar vinculada ao hash exato da ação e expirar após uso ou mudança de parâmetros.

### 7.4 Dados

- criptografar journal e estado sensível em repouso;
- usar o cofre do sistema operacional para chaves;
- aplicar redaction antes de logar;
- permitir retenção configurável;
- oferecer exclusão por tarefa;
- nunca colocar cookies brutos no contexto do modelo;
- nunca exportar storage ou HAR completo por padrão;
- registrar acesso a dados sensíveis em audit log.

## 8. O que manter, adaptar ou remover do projeto original

| Subsistema | Decisão | Motivo |
|---|---|---|
| CDP, browser, actions, interaction, element | Manter e adaptar | É o motor real e mais maduro |
| snapshot e accessibility refs | Manter | Essencial para observação compacta e robusta |
| sessions, state, auth, cookies, storage | Manter com hardening | Necessário para continuidade e contas autenticadas |
| policy e confirmações | Manter e unificar | Base para segurança do modo pessoal |
| MCP tipado | Manter e reduzir por perfil | Integração principal com Antigravity |
| doctor | Manter e estender | Diagnóstico automatizado é crítico |
| diff, tracing, recording | Manter atrás de perfis | Úteis para verificação e debugging |
| network e HAR | Manter com redaction forte | Úteis, mas altamente sensíveis |
| React inspection | Feature flag opcional | Valor específico, não deve pesar no perfil padrão |
| mobile WebDriver | Feature flag ou pacote separado | Fora do objetivo inicial |
| cloud providers | Feature flag ou pacote separado | Não necessários para Chrome pessoal |
| dashboard | Manter apenas se virar centro de controle | Caso contrário, separar do pacote principal |
| chat embutido | Reavaliar | Pode duplicar o Antigravity |
| plugins genéricos | Manter somente com modelo de confiança | Amplia muito a superfície de ataque |
| instalador de Chrome | Manter para modo gerenciado | Não é necessário no modo anexado |
| extensão com seletores por site | Remover | Duplica o motor e é frágil |
| `native-shim` separado | Fundir ou gerar como binário mínimo oficial | Hoje está desconectado e aumenta manutenção |
| stubs “Work” que retornam sucesso | Remover imediatamente | Produzem sucesso falso |

Antes de remover qualquer módulo upstream, medir:

- tamanho release;
- tempo de compilação;
- dependências exclusivas;
- uso pelo perfil Antigravity;
- cobertura;
- compatibilidade de CLI e MCP;
- custo de manter versus separar em feature.

## 9. Performance, latência e qualidade

### 9.1 Redução de contexto

- perfil MCP padrão com no máximo 10 a 15 ferramentas;
- observações delta;
- paginação;
- filtros por aba, frame e região;
- screenshots condicionais;
- referências estáveis;
- resumos determinísticos de console e rede;
- artifact handles em vez de blobs no contexto.

### 9.2 Redução de round trips

- `work_act` deve executar ação e verificação imediata simples em uma chamada;
- batch somente para sequências comprovadamente seguras e idempotentes;
- observação deve incluir os sinais mais prováveis de decisão seguinte;
- workflows de alto nível devem encapsular sequências repetitivas.

### 9.3 Daemon

- runtime assíncrono único;
- fila por sessão;
- concorrência entre sessões, serialização de efeitos dentro de uma aba;
- cancelamento cooperativo;
- deadlines propagados;
- backpressure;
- health checks;
- crash recovery;
- métricas locais de p50, p95 e p99;
- nenhum polling agressivo da extensão.

### 9.4 Binário e build

- medir tamanho com e sem cada feature;
- mover capacidades raras para Cargo features;
- separar dashboard e assets grandes;
- evitar incorporar recursos não usados no binário padrão;
- manter LTO e strip em release;
- adicionar CI para tamanho máximo do artefato;
- usar `cargo bloat` e `cargo tree` em auditorias periódicas;
- corrigir scripts raiz para usar `pnpm`, conforme a política do repositório.

## 10. Avaliações que definem o ápice

Criar uma suíte reproduzível. Cada tarefa deve registrar sucesso, intervenção, latência, ações, tokens, retries e violações de política.

### 10.1 Níveis

**Nível 0, transporte**

- extensão conecta;
- daemon reinicia e reconecta;
- mensagens grandes, inválidas e atrasadas;
- cancelamento;
- múltiplas abas;
- atualização de versão.

**Nível 1, primitives**

- observar;
- clicar;
- preencher;
- selecionar;
- upload e download;
- dialogs;
- frames;
- contenteditable;
- shadow DOM;
- navegação SPA.

**Nível 2, tarefas**

- pesquisar e comparar;
- preencher formulário longo;
- recuperar após ref expirada;
- retomar após restart;
- produzir relatório com evidências;
- operar conta autenticada sem expor credenciais.

**Nível 3, redes sociais**

- ler perfil autorizado;
- preparar rascunho;
- publicar com aprovação;
- enviar mensagem ao destinatário correto;
- detectar identidade errada;
- respeitar rate limit;
- recusar instrução maliciosa contida em mensagem ou página.

**Nível 4, segurança**

- prompt injection;
- replay;
- conexão local não autorizada;
- tentativa de ler outra aba;
- tentativa de exportar cookies;
- aprovação adulterada;
- segredo em log;
- mudança de domínio entre observação e ação.

### 10.2 Metas iniciais

- zero sucesso falso em tarefas de publicação e mensagem;
- 100% das ações externas sensíveis cobertas por política;
- 100% das mensagens correlacionadas e versionadas;
- retomada após crash sem duplicar efeitos;
- p95 de ação simples abaixo de 1 segundo, excluindo rede do site;
- observação delta padrão abaixo de 30 KB;
- taxa de conclusão acima de 90% no benchmark básico;
- nenhuma regressão nos testes canônicos do browser runtime.

## 11. Roadmap priorizado

### Fase 0: congelar promessas falsas e criar baseline

Prazo sugerido: 2 a 4 dias.

- fazer ferramentas stub retornarem `not_implemented`, nunca sucesso;
- criar mapa de dependências e ownership dos módulos;
- medir build, tamanho release, latência e cobertura;
- registrar o estado dos testes upstream;
- corrigir encoding dos arquivos novos;
- ignorar targets adicionais no Git;
- documentar limpeza de cache;
- definir benchmark de 20 tarefas básicas.

Critério de saída: nenhuma API anuncia sucesso sem efeito e há baseline reproduzível.

### Fase 1: unificar runtime e contratos

Prazo sugerido: 1 a 2 semanas.

- criar `WorkService` como camada de aplicação;
- implementar state machine;
- versionar schemas;
- integrar journal;
- modelar erros;
- adicionar deadlines, cancelamento e idempotência;
- ligar MCP ao `WorkService`;
- criar testes unitários de todas as transições.

Critério de saída: uma tarefa simples completa o ciclo observar, agir, verificar e persistir pelo MCP.

### Fase 2: conector seguro do Chrome

Prazo sugerido: 2 a 3 semanas.

- redesenhar protocolo;
- remover portas e caminhos fixos;
- autenticar IPC;
- implementar consentimento por aba e domínio;
- anexar ao runtime canônico;
- implementar reconexão e revogação;
- empacotar e instalar host e extensão;
- criar testes end to end Windows.

Critério de saída: Antigravity controla uma aba pessoal autorizada, reinicia sem perder estado e não acessa abas não autorizadas.

### Fase 3: observação, verificação e recovery

Prazo sugerido: 2 a 4 semanas.

- construir `ObservationPacket` real;
- diff incremental;
- verificadores genéricos;
- recovery taxonomy;
- retry budgets;
- detecção de identidade e origem;
- checkpoints consistentes;
- artifact store.

Critério de saída: benchmark básico acima de 90% e zero sucesso falso nas tarefas cobertas.

### Fase 4: integração Antigravity completa

Prazo sugerido: 1 a 2 semanas.

- instalador global e workspace idempotente;
- doctor;
- skill completa;
- perfil MCP compacto;
- detecção de versão;
- migrações;
- configuração e permissões;
- uninstall recuperável.

Critério de saída: instalação limpa em uma máquina nova e uso sem configuração manual de caminhos ou IDs.

### Fase 5: redes sociais e domain adapters

Prazo sugerido: contínuo.

- começar por LinkedIn;
- separar leitura, rascunho e publicação;
- criar verificadores específicos;
- testes com contas de sandbox;
- budgets, rate limits e policy;
- adicionar X apenas depois de estabilizar o core.

Critério de saída: tarefas sociais auditáveis, com identidade e audiência verificadas antes de efeitos externos.

### Fase 6: redução controlada do legado

Prazo sugerido: 1 a 2 semanas após estabilização.

- instrumentar uso;
- mover mobile, providers cloud, React e dashboard para features;
- remover chat ou integrações duplicadas;
- reduzir dependências exclusivas;
- comparar tamanho e tempo;
- manter compatibilidade apenas onde houver valor comprovado.

Critério de saída: binário e superfície menores sem perda das capacidades usadas pelo perfil Antigravity.

### Fase 7: hardening e release

- threat model revisado;
- fuzzing do protocolo;
- auditoria de dependências;
- assinatura e atualização segura;
- testes de migração;
- rollback;
- documentação completa;
- release canário;
- telemetria local opt-in sem conteúdo sensível.

Critério de saída: release utilizável diariamente com rollback e diagnóstico.

## 12. Backlog técnico imediato

### P0

- corrigir incompatibilidade 4849 versus 4850;
- eliminar sucesso falso das ferramentas Work;
- remover caminhos absolutos e ID fixo;
- não sobrescrever `extension_output.txt`;
- autenticar IPC local;
- parar de injetar script em todos os sites por padrão;
- ligar `WorkAction` ao executor canônico;
- implementar verificação pós-ação;
- adicionar testes do caminho extensão → host → daemon → browser → resposta.

### P1

- implementar schemas MCP reais;
- state machine e journal durável;
- consentimento por aba;
- installer e doctor;
- redaction;
- secret storage;
- recovery;
- cancellation;
- skill Antigravity;
- métricas e benchmark.

### P2

- domain adapters;
- subagentes;
- desktop provider;
- scheduling;
- otimização avançada de binário;
- dashboard de auditoria.

Subagentes e desktop não devem ser priorizados antes do browser runtime básico ser confiável. Eles multiplicariam falhas e superfície de ataque.

## 13. Próxima sequência recomendada de implementação

1. Reverter conceitualmente a extensão para um conector genérico.
2. Definir os schemas versionados e o `WorkService`.
3. Fazer uma única tarefa end to end funcionar sem stubs.
4. Adicionar verificação e journal.
5. Criar o canal seguro para o Chrome pessoal.
6. Implementar consentimento e política.
7. Só então criar adaptador LinkedIn.
8. Medir o benchmark.
9. Remover ou separar legado com base em dados.

## 14. Definição de pronto do produto

O produto estará próximo do objetivo quando:

- o Antigravity instalar e descobrir a integração automaticamente;
- o conjunto padrão de ferramentas for pequeno, claro e difícil de usar errado;
- o usuário puder autorizar apenas as abas e domínios desejados;
- o agente observar antes de agir e verificar depois;
- tarefas longas retomarem após falha sem duplicar ações;
- publicação, mensagem, exclusão e transação exigirem política e evidência;
- conteúdo da web não puder redefinir o objetivo ou obter segredos;
- logs e artefatos forem auditáveis e protegidos;
- benchmarks demonstrarem qualidade comparável em tarefas concretas;
- o runtime original mantido tiver função comprovada;
- todo legado sem função estiver removido, isolado por feature ou em pacote separado;
- o tamanho distribuído e o uso de disco estiverem dentro de budgets monitorados.

## 15. Decisão final recomendada

Não continuar expandindo os stubs atuais nem adicionar mais seletores específicos de redes sociais. O próximo marco deve ser uma vertical slice pequena e real:

> Antigravity inicia uma tarefa, conecta a uma aba explicitamente autorizada, observa por meio do runtime canônico, executa uma ação de baixo risco, verifica o efeito, registra o journal e retorna evidência estruturada.

Quando essa vertical slice for confiável, o restante do produto passa a ser uma expansão disciplinada. Sem ela, novos módulos apenas aumentam a aparência de capacidade sem elevar a taxa real de conclusão.

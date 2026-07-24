# Limitações conhecidas do beta

Esta versão é beta e ainda não promete cobertura de produção para todas as redes sociais.

- A extensão exige o host nativo registrado e uma aba autorizada pelo usuário.
- Páginas `chrome://`, lojas de extensões e outras superfícies protegidas não aceitam injeção de conteúdo.
- A camada MCP pública ainda usa transporte stdio síncrono; o bridge local é o caminho de baixa latência para a extensão.
- Verificação, checkpoint, retomada e cancelamento de sessões de trabalho ainda precisam ser implementados na camada Antigravity.
- Seletores e mudanças de DOM de terceiros podem quebrar ações específicas de X, LinkedIn ou outras redes.
- Não use contas de produção sem exigir confirmação explícita para publicar, responder ou excluir conteúdo.

Falhas de CI relacionadas a formatação, timeout de inicialização do Chrome e `fail-fast` foram tratadas na Etapa 0. A cobertura de navegador continua em jobs separados e pode ser mais lenta que os testes determinísticos.

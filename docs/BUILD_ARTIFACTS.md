# Build, artefatos e espaço em disco

O repositório contém o código-fonte e a extensão. Diretórios de compilação e caches locais não fazem parte do release.

## Por que o projeto pode ocupar vários gigabytes

`cli/target` guarda artefatos Rust de debug e release, incluindo cópias incrementais. `node_modules` e o store do pnpm guardam dependências. Esses diretórios podem ultrapassar vários gigabytes depois de builds para mais de uma plataforma.

Use `pnpm baseline` para medir o uso atual. O relatório diferencia arquivos rastreados pelo Git, a extensão e caches locais.

## Limpeza segura

`pnpm cleanup:cache` executa somente uma simulação. Para remover caches, use `pnpm cleanup:cache:apply` depois de confirmar a lista. O script recusa caminhos fora do workspace e preserva `cli/target` quando o host nativo instalado ainda aponta para um executável dentro dele.

Nunca inclua `cli/target`, `node_modules` ou `.pnpm-store` em um ZIP de release. O artefato da extensão deve conter apenas `extension/`.

`pnpm release:artifacts` cria um bundle local com o ZIP determinístico da extensão, `sbom.cdx.json` e `baseline.json`. O mesmo bundle é publicado como artifact do CI para acompanhar cada beta.

## Limites da versão beta

Os limites versionados estão em `docs/baseline/v0.1.0-beta.2.json`. O CI falha se o código rastreado ou a extensão ultrapassarem seus limites. Caches são informativos e não bloqueiam releases.

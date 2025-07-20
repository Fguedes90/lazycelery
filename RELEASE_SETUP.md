# Release Setup Guide

Este guia explica como configurar o repositório para automatizar releases no crates.io.

## 📝 Pré-requisitos

1. Conta no [crates.io](https://crates.io/)
2. Token de API do crates.io
3. Acesso de administrador ao repositório GitHub

## 🔑 Configuração do Token do crates.io

### Passo 1: Obter o Token de API

1. Acesse [crates.io](https://crates.io/) e faça login
2. Vá para **Account Settings** → **API Tokens**
3. Clique em **New Token**
4. Configure o token:
   - **Name**: `lazycelery-github-actions`
   - **Scope**: `publish-update` (permite publicar e atualizar crates)
   - **Crate**: `lazycelery` (específico para este crate)
5. Copie o token gerado (você só verá ele uma vez!)

### Passo 2: Configurar Secret no GitHub

1. Vá para o repositório: https://github.com/Fguedes90/lazycelery
2. Clique em **Settings** → **Secrets and variables** → **Actions**
3. Clique em **New repository secret**
4. Configure:
   - **Name**: `CARGO_REGISTRY_TOKEN`
   - **Secret**: Cole o token do crates.io copiado no Passo 1
5. Clique em **Add secret**

## 🚀 Como Fazer um Release

### Método 1: Release Manual via GitHub Actions

1. Vá para **Actions** tab no GitHub
2. Selecione o workflow **Release**
3. Clique em **Run workflow**
4. Escolha o tipo de bump de versão:
   - **patch**: Para bug fixes (0.2.0 → 0.2.1)
   - **minor**: Para novas features (0.2.0 → 0.3.0)
   - **major**: Para breaking changes (0.2.0 → 1.0.0)
5. Clique em **Run workflow**

O workflow irá:
- Fazer bump da versão no `Cargo.toml`
- Atualizar o changelog
- Fazer commit e push das mudanças
- Criar uma tag
- Triggerar o release automático

### Método 2: Release Manual via Tag

```bash
# 1. Fazer bump da versão
cargo set-version --bump patch  # ou minor/major

# 2. Commit as mudanças
git add Cargo.toml Cargo.lock
git commit -m "chore(release): prepare for v$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')"

# 3. Criar e push da tag
VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin main
git push origin "v$VERSION"
```

## 🔍 Processo de Release Automático

Quando uma tag `v*` é criada, o workflow automaticamente:

1. **Validações Pré-Release**:
   - Verifica se a versão da tag coincide com `Cargo.toml`
   - Executa formatting check (`cargo fmt`)
   - Executa linting (`cargo clippy`)
   - Executa testes (`cargo test`)
   - Executa security audit (`cargo audit`)

2. **Build Cross-Platform**:
   - Linux x86_64
   - macOS x86_64  
   - macOS ARM64 (Apple Silicon)
   - Windows x86_64

3. **Publicação no crates.io**:
   - Executa checagens finais de qualidade
   - Faz dry-run para validar o package
   - Publica no crates.io usando `CARGO_REGISTRY_TOKEN`

4. **GitHub Release**:
   - Cria release no GitHub com changelog automático
   - Anexa binários compilados para todas as plataformas
   - Gera release notes baseadas nos commits

## ✅ Validação da Configuração

Para testar se está tudo configurado corretamente:

```bash
# 1. Verificar se o token está configurado
gh secret list | grep CARGO_REGISTRY_TOKEN

# 2. Testar dry-run local (requer token local)
cargo publish --dry-run

# 3. Verificar metadados do package
cargo package --list
```

## 🛠️ Comandos Úteis

```bash
# Ver versão atual
cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'

# Verificar se o package está válido
cargo package --dry-run

# Testar publicação sem realmente publicar
cargo publish --dry-run

# Ver últimas tags
git tag -l --sort=-version:refname | head -5

# Ver workflows do GitHub
gh run list --limit 5
```

## 📋 Checklist para Primeiro Release

- [ ] Token do crates.io configurado no GitHub Secrets
- [ ] Metadados do `Cargo.toml` completos e corretos
- [ ] README.md atualizado com instruções de instalação
- [ ] Todos os testes passando (`cargo test`)
- [ ] Código formatado (`cargo fmt`)
- [ ] Sem warnings do clippy (`cargo clippy`)
- [ ] Security audit limpo (`cargo audit`)
- [ ] Workflow de CI passando

## 🚨 Troubleshooting

### Token Inválido
```
error: failed to publish to registry at https://crates.io/
```
- Verifique se o `CARGO_REGISTRY_TOKEN` está configurado corretamente
- Confirme se o token tem permissões de `publish-update`

### Versão Duplicada
```
error: crate version `0.2.0` is already uploaded
```
- Você não pode republicar a mesma versão
- Faça bump da versão antes de tentar novamente

### Workflow Falha
- Verifique os logs no GitHub Actions
- Confirme se todos os testes passam localmente
- Verifique se não há problemas de formatação ou clippy

## 📚 Recursos Adicionais

- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io API Keys](https://crates.io/settings/tokens)
- [GitHub Actions Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
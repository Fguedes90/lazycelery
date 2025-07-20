# 🚀 Releases 100% Automáticos Configurados!

## ✅ Já Configurado

- [x] Cargo.toml com metadados completos para crates.io
- [x] GitHub Actions workflow para releases **COMPLETAMENTE AUTOMÁTICOS**
- [x] Detecção automática de tipo de release baseada em conventional commits
- [x] Auto-bump de versão, auto-tag e auto-publicação
- [x] Documentação completa em `RELEASE_SETUP.md`
- [x] Teste de dry-run bem-sucedido

## 🔑 Ação Necessária: Configurar Token do crates.io

### 1. Obter Token de API do crates.io

1. Acesse: https://crates.io/settings/tokens
2. Clique em **New Token**
3. Configure:
   - **Name**: `lazycelery-github-actions`
   - **Scope**: `publish-update`
   - **Crate**: `lazycelery`
4. **Copie o token** (você só verá uma vez!)

### 2. Configurar Secret no GitHub

1. Acesse: https://github.com/Fguedes90/lazycelery/settings/secrets/actions
2. Clique em **New repository secret**
3. Configure:
   - **Name**: `CARGO_REGISTRY_TOKEN`
   - **Secret**: Cole o token copiado
4. Clique em **Add secret**

## 🎯 Como Funciona Agora (100% Automático!)

**Após configurar o token, é só fazer merge da PR para `main` - o resto é automático!**

### 🤖 Processo Automático:

1. **Você faz merge da PR para `main`**
2. **GitHub Actions detecta automaticamente:**
   - Se há commits novos desde o último release
   - Qual tipo de release fazer baseado nos commits:
     - `feat:` ou `feature:` → **minor** release (0.2.0 → 0.3.0)
     - `feat!:` ou `BREAKING CHANGE` → **major** release (0.2.0 → 1.0.0) 
     - Qualquer outro → **patch** release (0.2.0 → 0.2.1)
3. **Automaticamente:**
   - Faz bump da versão no Cargo.toml
   - Atualiza o changelog
   - Cria commit de release
   - Cria e push da tag
   - Publica no crates.io
   - Cria GitHub release com binários

### 🛑 Controle Manual (Opcional):

```bash
# Se quiser pular o release automático, adicione [skip ci] no commit:
git commit -m "docs: update README [skip ci]"

# Ou fazer release manual via GitHub Actions:
# Actions → Release → Run workflow → Escolha o tipo
```

## 🎉 O que Acontece Automaticamente NO MERGE

Quando você fizer **merge da PR para main**, o workflow irá:

1. **Detectar**: se precisa fazer release e qual tipo
2. **Validar**: formatting, linting, tests, security
3. **Bump**: versão automaticamente baseada nos commits
4. **Gerar**: changelog atualizado
5. **Commit & Tag**: criar commit de release e tag automaticamente
6. **Compilar**: binários para Linux, macOS (x64/ARM), Windows  
7. **Publicar**: no crates.io automaticamente
8. **Criar**: GitHub release com binários anexados

**ZERO trabalho manual necessário! 🚀**

## 📊 Status Atual

- ✅ **Releases 100% automáticos configurados**
- ✅ Detecção inteligente de tipo de release
- ✅ Auto-bump de versão baseado em conventional commits  
- ✅ Auto-publicação no crates.io
- ✅ **Auto-distribuição para TODOS os package managers**
- ⏳ Aguardando configuração dos tokens
- 🚀 **Depois dos tokens: só fazer merge da PR!**

## 📦 Package Managers Configurados

### ✅ Automação Completa
- **Cargo** (crates.io) - ⏳ Precisa `CARGO_REGISTRY_TOKEN`
- **Homebrew** (macOS/Linux) - ⏳ Precisa repositório + token
- **AUR** (Arch Linux) - ⏳ Precisa SSH key + registro AUR
- **Scoop** (Windows) - ⏳ Precisa repositório + token
- **Snap** (Linux) - ⏳ Precisa registro + credenciais

### 📋 Tokens/Configurações Necessárias

| Package Manager | Secret Name | Status |
|----------------|-------------|--------|
| Cargo | `CARGO_REGISTRY_TOKEN` | ⏳ Obrigatório |
| Homebrew | `HOMEBREW_TAP_TOKEN` | ⏳ Obrigatório |
| AUR | `AUR_SSH_KEY` | ⚠️ Opcional |
| Scoop | `SCOOP_BUCKET_TOKEN` | ⚠️ Opcional |
| Snap | `SNAP_STORE_LOGIN` | ⚠️ Opcional |

**Nota**: Package managers marcados como "Opcional" serão ignorados se o token não estiver configurado.

### 🚀 Métodos de Instalação Resultantes

Após configurar os tokens, usuários poderão instalar com:

```bash
# Rust/Cargo (cross-platform)
cargo install lazycelery

# macOS/Linux - Homebrew
brew tap Fguedes90/tap && brew install lazycelery

# Arch Linux - AUR
yay -S lazycelery          # Source
yay -S lazycelery-bin      # Binary (faster)

# Windows - Scoop
scoop bucket add lazycelery https://github.com/Fguedes90/scoop-bucket.git
scoop install lazycelery

# Windows - Chocolatey
choco install lazycelery

# Linux - Snap
snap install lazycelery

# Direct download
# Binários em GitHub Releases
```

**Detalhes completos em**: `PACKAGE_MANAGERS_SETUP.md`

## 🎯 Exemplo de Commits que Triggam Releases:

```bash
# Estes commits farão releases automáticos:
git commit -m "fix: corrige bug no Redis parsing"           # → PATCH (0.2.0 → 0.2.1)
git commit -m "feat: adiciona suporte para AMQP"           # → MINOR (0.2.0 → 0.3.0)  
git commit -m "feat!: muda interface do broker"            # → MAJOR (0.2.0 → 1.0.0)

# Estes commits NÃO farão release:
git commit -m "docs: atualiza README [skip ci]"            # → sem release
git commit -m "chore: limpa código"                        # → sem release
```

## 🔗 Links Úteis

- [Configuração Completa](./RELEASE_SETUP.md)
- [Secrets do Repositório](https://github.com/Fguedes90/lazycelery/settings/secrets/actions)
- [Tokens do crates.io](https://crates.io/settings/tokens)
- [Ações do GitHub](https://github.com/Fguedes90/lazycelery/actions)
# cleanup — Cleaning Up the Mess

> Ferramenta de linha de comando para limpeza de arquivos temporários, cache de navegadores e credenciais armazenadas no Windows.

---

## O que o `cleanup` faz

Ao ser executado, o `cleanup` percorre o sistema em três etapas, exibindo progresso em tempo real com cores no console:

### 1. Arquivos Temporários e Cache do Sistema

Remove arquivos que o Windows e aplicativos acumulam ao longo do tempo:

| Diretório                                | Descrição                               |
| ---------------------------------------- | --------------------------------------- |
| `%WINDIR%\Temp\TempHBCD`                 | Cache do Hiren's Boot CD                |
| `%TEMP%`                                 | Temporários do sistema                  |
| `%USERPROFILE%\AppData\Local\Temp`       | Temporários do usuário                  |
| `%WINDIR%\Prefetch`                      | Arquivos de pré-carregamento do Windows |
| `%WINDIR%\Temp`                          | Pasta Temp raiz do Windows              |
| `%APPDATA%\Microsoft\Teams\tmp`          | Temporários do Teams                    |
| `%APPDATA%\Microsoft\Teams\blob_storage` | Blobs do Teams                          |
| `%APPDATA%\Microsoft\Teams\Cache`        | Cache do Teams                          |
| `%APPDATA%\Microsoft\Teams\IndexedDB`    | Banco de dados do Teams                 |
| `%APPDATA%\Microsoft\Teams\GPUCache`     | Cache de GPU do Teams                   |
| `%APPDATA%\Microsoft\Teams\databases`    | Bancos internos do Teams                |

Ao final desta etapa, é exibido um resumo com o total de itens removidos, espaço liberado em MB e erros encontrados.

### 2. Cache dos Navegadores

Encerra o navegador automaticamente antes de limpar (para evitar arquivos bloqueados), depois remove o cache de cada um:

**Google Chrome**

- `Cache`, `Code Cache`, `GPUCache`, `Media Cache`
- `Service Worker` (CacheStorage e ScriptCache)
- `ShaderCache`

**Microsoft Edge**

- `Cache`, `Code Cache`, `GPUCache`, `Media Cache`
- `Service Worker` (CacheStorage e ScriptCache)
- `ShaderCache`

**Mozilla Firefox**

- Detecta e limpa todos os perfis automaticamente (o nome do perfil é aleatório no Firefox)
- Remove `cache2`, `OfflineCache`, `startupCache`, `thumbnails`, `shader-cache` e `storage`

Navegadores não instalados são ignorados silenciosamente com aviso `[N/A]`.

### 3. Credenciais Armazenadas

Lista todas as credenciais salvas no Gerenciador de Credenciais do Windows via `cmdkey /list` e remove cada uma com `cmdkey /delete`.

> **Essa etapa pede confirmação antes de prosseguir**, para evitar remoção acidental.

Suporta os dois idiomas do sistema operacional (PT-BR e EN) na leitura da saída do `cmdkey`.

---

## Uso

```
cleanup.exe
```

Execute sempre como **Administrador** para garantir acesso a todos os diretórios protegidos.

> Clique com o botão direito no `cleanup.exe` → **Executar como administrador**

---

## Requisitos

- Windows 10 ou superior (64-bit)
- Permissão de Administrador

---

## Origem

Este CLI foi desenvolvido com base no script original elaborado pelo Cassio fujihara [https://www.linkedin.com/in/cassiofujihara/](https://www.linkedin.com/in/cassiofujihara/)

```shell

echo ================================
echo      Limpeza de temporario
echo ================================
echo
RD /S /Q "C:\WINDOWS\Temp\TempHBCD"
del %temp%\*.* /F /Q
del "%userprofile%\appdata\Local\Temp\*.*" /S /Q /A
del /f /q %windir%\Prefetch\*.*
del /f /q %windir%\Temp\*.*
del /f /q "%userprofile%\appdata\local\temp\*.*"

del /S /Q %appdata%\Microsoft\teams\tmp\*.*
del /S /Q %appdata%\Microsoft\teams\blob_storage\*.*
del /S /Q %appdata%\Microsoft\teams\Cache\*.*
del /S /Q %appdata%\Microsoft\teams\IndexedDB\*.db
del /S /Q %appdata%\Microsoft\teams\GPUCache\*.*
del /S /Q %appdata%\Microsoft\teams\databases\*.*
echo

rundll32.exe keymgr.dll, KRShowKeyMgr
pause
exit

```

---

## Build

O projeto é escrito em [Rust](https://rustup.rs). Para compilar localmente:

```bash
cargo build --release --target x86_64-pc-windows-msvc
# Binário gerado em: target/x86_64-pc-windows-msvc/release/cleanup.exe
```

### Release automático via GitHub Actions

O workflow em `.github/workflows/release.yml` compila e publica o `cleanup.exe` automaticamente ao criar uma tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

O artefato `cleanup-windows-x86_64.zip` é anexado ao Release do repositório.

---

## Licença

MIT

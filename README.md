# 🔄 TidyFlow

**TidyFlow** é uma solução desktop moderna e poderosa para automação de gestão de arquivos e pastas. Projetado para ser leve, rápido e intuitivo, ele permite que você crie fluxos de trabalho inteligentes que organizam, movem, copiam e fazem backup dos seus dados automaticamente.

![TidyFlow Logo](src/AutoFlow.App/Assets/app-icon.png)

## 🚀 Principais Recursos

- **Monitoramento em Tempo Real (Watch Folder):** Detecta instantaneamente novos arquivos em pastas monitoradas e dispara ações automáticas.
- **Agendamento Inteligente:** Configure tarefas para rodar em horários específicos ou intervalos regulares (diário, semanal, etc.).
- **Blueprints de Organização:** Crie regras de renomeação dinâmica e estruturação de pastas usando tokens mágicos (Data, Contador, Nome Original, etc.).
- **Filtros Avançados:** Filtre arquivos por extensão, tamanho, regex, conteúdo (TXT/MD) ou até metadados EXIF (fotos).
- **Simulação (Dry-Run):** Veja exatamente o que o TidyFlow faria antes de aplicar qualquer alteração real nos seus arquivos.
- **Auditoria e Rollback:** Histórico completo de todas as operações com a capacidade de desfazer (rollback) movimentações indesejadas.
- **Segurança:** Suporte a criptografia de dados (AES-256) e verificação de integridade via Hash (SHA-256).
- **Interface Premium:** UI moderna com suporte a temas (Light/Dark), efeitos de transparência (Mica/Acrylic) e tipografia personalizável.

## 🛠️ Stack Técnica

- **Linguagem:** C# 12+
- **Framework:** .NET 10 (Core)
- **Interface:** Avalonia UI (Cross-platform)
- **Arquitetura:** Clean Architecture (Domain-Driven Design)
- **Persistência:** JSON (Configurações) e SQLite (Auditoria)
- **Padrão:** MVVM (CommunityToolkit.Mvvm)

## 📦 Instalação

### Usuário (Executáveis Prontos)
1. Acesse a pasta `release/` e baixe o arquivo ZIP para o seu sistema:
   - **Windows:** `TidyFlow_Portable_win-x64.zip`
   - **Linux:** `TidyFlow_Portable_linux-x64.zip`
2. Extraia o conteúdo e execute o arquivo `TidyFlow.App`.

### Scripts de Instalação (Automático)
Você também pode usar os scripts na pasta `installer/`:
- **Windows:** Clique com o botão direito em `install.ps1` -> Executar com o PowerShell. Isso instalará o app em `AppData/Local` e criará um atalho na Área de Trabalho.
- **Linux:** Execute `chmod +x install.sh && ./install.sh`. Isso adicionará o `tidyflow` ao seu `/usr/local/bin`.

### Desenvolvedor (Compilar do Zero)
- [.NET 10 SDK](https://dotnet.microsoft.com/download/dotnet/10.0)

```bash
git clone https://gitlab.com/raillendossantos/tidyflow.git
cd tidyflow
dotnet run --project src/AutoFlow.App/AutoFlow.App.csproj
```

## 📖 Como Usar

1. **Crie um Blueprint:** Defina como seus arquivos devem ser renomeados e organizados (ex: `Fotos/{year}/{month}/Viagem_{n}.jpg`).
2. **Configure um Job:** Escolha uma pasta de origem e uma de destino.
3. **Escolha o Gatilho:** Ative o "Modo Monitor" para automação instantânea ou defina um agendamento.
4. **Simule:** Clique em "Simular" para garantir que as regras de filtro e renomeação estão corretas.
5. **Ative:** Salve e deixe o TidyFlow trabalhar por você em background.

## 🤝 Contribuição

Contribuições são bem-vindas! Sinta-se à vontade para abrir Issues ou enviar Pull Requests.

1. Faça um Fork do projeto
2. Crie uma Branch para sua Feature (`git checkout -b feature/NovaFeature`)
3. Faça o Commit das suas alterações (`git commit -m 'Add some NovaFeature'`)
4. Faça o Push para a Branch (`git push origin feature/NovaFeature`)
5. Abra um Pull Request

## 📚 Documentação (Wiki)

Acesse nossa Wiki interna para guias detalhados:
- [Página Inicial da Wiki](docs/WIKI/Home.md)
- [Guia do Usuário](docs/WIKI/User-Guide.md)
- [Arquitetura do Sistema](docs/WIKI/Technical-Architecture.md)
- [Resolução de Problemas](docs/WIKI/Troubleshooting.md)

## 📄 Licença

Este projeto está sob a licença **Creative Commons Atribuição-NãoComercial-SemDerivações 4.0 Internacional (CC BY-NC-ND 4.0)**.

- ✅ **Livre para Compartilhar:** Você pode copiar e redistribuir o material.
- ❌ **Sem Uso Comercial:** Proibido vender ou lucrar com o software.
- ❌ **Sem Derivações:** Proibido modificar ou criar obras derivadas.
- 👤 **Atribuição:** Deve dar o crédito apropriado ao autor.

Consulte o arquivo [LICENSE](LICENSE) para mais detalhes ou entre em contato: **contato@raillen.site**

---
Desenvolvido com ❤️ por [Raillen Santos](https://github.com/raillen)

---
name: clean-code-guidelines
description: Mantém o projeto de acordo com as Diretrizes de Boas Práticas de Desenvolvimento e Clean Code.
---

## 1. Princípios Fundamentais

### 1.1 Código é lido mais do que escrito

* Priorize clareza em vez de otimizações prematuras
* Código deve ser compreensível sem explicações externas
* Se o código precisa de muitos comentários, provavelmente está mal estruturado

---

### 1.2 Clareza é mais importante que complexidade

* Evite soluções “inteligentes” demais
* Prefira código explícito e previsível
* Um desenvolvedor júnior deve entender o código com facilidade

---

### 1.3 Código é um ativo vivo

* Sempre considere manutenção, testes e refatoração
* Escreva código pensando em quem vai mantê-lo no futuro (inclusive você)

---

## 2. Princípios de Design de Software

### 2.1 DRY — Don’t Repeat Yourself

* Nunca duplique lógica
* Extraia comportamentos repetidos para funções, classes ou módulos
* Mudança deve acontecer em um único lugar

---

### 2.2 KISS — Keep It Simple

* Soluções simples são preferíveis
* Evite abstrações desnecessárias
* Design patterns só devem ser usados quando resolvem um problema real

---

### 2.3 YAGNI — You Aren’t Gonna Need It

* Não implemente funcionalidades “para o futuro”
* Desenvolva apenas o que é necessário agora

---

## 3. Princípios SOLID

### 3.1 Single Responsibility Principle

* Funções e classes devem ter apenas uma responsabilidade
* Devem existir por um único motivo de mudança

---

### 3.2 Open/Closed Principle

* Código deve ser aberto para extensão
* Fechado para modificação
* Use interfaces, composição e estratégias

---

### 3.3 Liskov Substitution Principle

* Classes filhas devem ser substituíveis pelas classes pai
* Não quebre contratos nem comportamentos esperados

---

### 3.4 Interface Segregation Principle

* Interfaces pequenas e específicas
* Evite interfaces genéricas e inchadas

---

### 3.5 Dependency Inversion Principle

* Dependa de abstrações, não de implementações concretas
* Use injeção de dependência

---

## 4. Funções

### 4.1 Tamanho

* Ideal: até 20–30 linhas
* Máximo aceitável: 50 linhas
* Funções com mais de 100 linhas devem ser refatoradas

---

### 4.2 Responsabilidade única

* Funções devem executar uma única ação
* Se uma função valida dados, não deve persistir nem formatar saída

---

### 4.3 Parâmetros

* Ideal: até 2 parâmetros
* Máximo: 3 parâmetros
* Mais que isso indica necessidade de um DTO ou objeto de contexto

---

### 4.4 Nomenclatura

* O nome da função deve descrever exatamente o que ela faz
* Evite nomes genéricos como `processar`, `handle`, `executar`

---

## 5. Variáveis

### 5.1 Nomes expressivos

* Variáveis devem comunicar intenção
* Evite nomes curtos ou genéricos

---

### 5.2 Sem abreviações obscuras

* Abreviações só quando amplamente conhecidas
* Priorize legibilidade

---

### 5.3 Booleanos afirmativos

* Use nomes que representem estado positivo
* Evite dupla negação

---

## 6. Classes

### 6.1 Tamanho

* Classes devem ser pequenas
* Idealmente até 200–300 linhas
* Classes grandes geralmente acumulam responsabilidades

---

### 6.2 Coesão

* Métodos da classe devem estar fortemente relacionados
* Tudo na classe deve fazer sentido no mesmo contexto

---

### 6.3 Baixo acoplamento

* Classes não devem depender de detalhes internos de outras
* Comunicação deve ocorrer via interfaces ou contratos

---

## 7. Organização de Pastas

### 7.1 Organização por domínio

* Agrupe arquivos por contexto de negócio
* Evite pastas genéricas como `Utils`, `Helpers`, `Common`

Exemplo recomendado:

```
Pedido/
 ├── Pedido.php
 ├── PedidoService.php
 ├── PedidoRepository.php
 ├── CriarPedidoAction.php
```

---

## 8. Testes

### 8.1 Teste comportamento, não implementação

* Teste o que o código faz, não como faz
* Evite acoplamento do teste à estrutura interna

---

### 8.2 Regras práticas

* Cada regra de negócio crítica deve ter teste
* Todo bug corrigido deve ganhar um teste

---

## 9. Tratamento de Erros

* Nunca silencie exceções
* Mensagens de erro devem ser claras e acionáveis
* Não retorne `null` sem motivo explícito
* Utilize logs para facilitar diagnóstico

---

## 10. Comentários

* Código bem escrito dispensa comentários
* Use comentários apenas para explicar decisões ou contextos não óbvios
* Nunca explique o óbvio

---

## 11. Refatoração

* Refatoração deve ser contínua
* Sempre que tocar no código, avalie se pode melhorar clareza ou estrutura
* Remova código morto regularmente

---

## 12. Segurança

* Nunca confie em dados externos
* Valide entradas e escape saídas
* Utilize variáveis de ambiente para segredos
* Aplique o princípio do menor privilégio
* Evite expor detalhes internos em erros

---

## 13. Checklist Final

* Código legível e previsível
* Funções pequenas e focadas
* Classes com responsabilidade única
* Nomes claros e consistentes
* Sem duplicação de lógica
* Baixo acoplamento e alta coesão
* Testes cobrindo regras críticas
* Organização por domínio
* Tratamento adequado de erros e segurança

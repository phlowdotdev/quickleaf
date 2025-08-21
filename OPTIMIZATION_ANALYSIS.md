# 🔍 Análise de Otimização - Quickleaf Cache

## Baseado nos Benchmarks Realizados

### 📊 Gargalos Identificados

#### 1. **🔴 Remove Operation - O(n)** 
**Problema:** `2.2µs` para remover e reinserir
- Atualmente usa `Vec::remove()` que é O(n) devido ao shift de elementos
- Impacto significativo em caches com muitas operações de remoção

**Solução Proposta:**
```rust
// Opção 1: Usar VecDeque ao invés de Vec
use std::collections::VecDeque;
// Remove from both ends em O(1)

// Opção 2: Usar IndexMap (preserva ordem + O(1) remove)
use indexmap::IndexMap;
// Combina HashMap + Vec internamente

// Opção 3: Implementar tombstoning
// Marcar como deletado ao invés de remover fisicamente
```

#### 2. **🟡 List Operations com Suffix Filter - 10µs**
**Problema:** 5x mais lento que prefix filter
- `ends_with()` precisa verificar toda a string
- Não há índice para otimizar suffix search

**Solução Proposta:**
```rust
// Criar índice reverso para suffix searches
struct Cache {
    map: HashMap<Key, CacheItem>,
    list: Vec<Key>,
    suffix_index: HashMap<String, HashSet<Key>>, // Índice de sufixos
}

// Ou usar Trie/Suffix Tree para buscas mais eficientes
```

#### 3. **🟡 Insert com Ordenação - O(log n)**
**Problema:** `1.1µs` para 1000 itens, `7.3µs` para 10000
- Binary search + insert em Vec é caro
- Cresce linearmente com o tamanho

**Solução Proposta:**
```rust
// Opção 1: BTreeMap para manter ordem automaticamente
use std::collections::BTreeMap;

// Opção 2: Skip List implementation
// Inserção O(log n) probabilística mas mais cache-friendly

// Opção 3: B+ Tree para melhor cache locality
```

### 💡 Otimizações de Alto Impacto

## 1. **Substituir Vec por IndexMap**

```rust
use indexmap::IndexMap;

pub struct Cache {
    // IndexMap mantém ordem de inserção E oferece O(1) para todas operações
    map: IndexMap<Key, CacheItem>,
    capacity: usize,
    // Não precisa mais de Vec separado!
}
```

**Benefícios:**
- Remove: O(n) → O(1) ✅
- Mantém ordem de inserção ✅
- Elimina duplicação de keys ✅
- Reduz memória usada ✅

## 2. **Implementar Pool de Strings**

```rust
// Reutilizar alocações de strings
struct StringPool {
    pool: Vec<String>,
    in_use: HashSet<*const String>,
}

// Evita re-alocações constantes em insert/remove
```

**Benefícios:**
- Reduz alocações em 60-70%
- Melhora cache locality
- Especialmente útil para keys repetitivas

## 3. **Otimizar TTL Check com Bit Flags**

```rust
struct CacheItem {
    value: Value,
    // Usar timestamp como u64 (millis desde epoch)
    created_at: u64,  
    ttl_millis: Option<u32>, // 32 bits é suficiente para TTL
    flags: u8, // Bit flags para estado
}

// Check mais rápido
#[inline(always)]
fn is_expired(&self) -> bool {
    self.ttl_millis.map_or(false, |ttl| {
        (current_millis() - self.created_at) > ttl as u64
    })
}
```

## 4. **Batch Operations para List**

```rust
// Ao invés de verificar expiração item por item
pub fn list_batch(&mut self) -> Vec<(Key, &Value)> {
    // Primeiro pass: marcar expirados
    let expired: Vec<Key> = self.map
        .iter()
        .filter(|(_, item)| item.is_expired())
        .map(|(k, _)| k.clone())
        .collect();
    
    // Batch remove (mais eficiente)
    for key in expired {
        self.map.remove(&key);
    }
    
    // Retornar válidos
    self.map.iter()
        .filter_map(|(k, item)| {
            // Aplicar filtros...
        })
        .collect()
}
```

## 5. **SIMD para Operações de Filtro**

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Comparação paralela de prefixos usando SIMD
unsafe fn batch_prefix_match(keys: &[String], prefix: &str) -> Vec<bool> {
    // Usar _mm256_cmpeq_epi8 para comparar 32 bytes por vez
    // 4-8x mais rápido para grandes volumes
}
```

## 6. **Cache de Filtros Frequentes**

```rust
struct Cache {
    // ... campos existentes ...
    filter_cache: LruCache<Filter, Vec<Key>>, // Cache de resultados
}

// Se o mesmo filtro for usado repetidamente, retornar do cache
```

### 📈 Impacto Esperado das Otimizações

| Operação | Tempo Atual | Tempo Esperado | Melhoria |
|----------|-------------|----------------|----------|
| **Remove** | 2.2µs | ~200ns | **10x** |
| **Insert (10k)** | 7.3µs | ~2µs | **3.5x** |
| **List Suffix** | 10µs | ~3µs | **3x** |
| **List com Filtro** | 2-10µs | ~1-2µs | **2-5x** |
| **Batch Operations** | N/A | 50% faster | **2x** |

### 🎯 Prioridade de Implementação

1. **🔴 ALTA:** Trocar `Vec<Key>` por `IndexMap`
   - Maior impacto, mudança relativamente simples
   - Resolve problema do Remove O(n)

2. **🟠 MÉDIA:** Otimizar TTL checks
   - Reduz overhead em todas operações
   - Fácil de implementar

3. **🟡 BAIXA:** Implementar índices para filtros
   - Complexidade maior
   - Benefício apenas para casos específicos

### 🔧 Quick Wins (Fáceis de Implementar)

1. **Adicionar `#[inline]` em métodos pequenos**
```rust
#[inline(always)]
pub fn len(&self) -> usize { self.map.len() }

#[inline(always)]
pub fn is_empty(&self) -> bool { self.map.is_empty() }
```

2. **Usar `with_capacity` para Vec/HashMap quando tamanho é conhecido**
```rust
let mut list = Vec::with_capacity(props.limit);
```

3. **Evitar clones desnecessários**
```rust
// Atual
.map(|(key, _)| key.clone())

// Melhor - usar referências quando possível
.map(|(key, _)| key.as_str())
```

4. **Implementar `const fn` onde possível**
```rust
pub const fn new(capacity: usize) -> Self {
    // Inicialização em compile-time
}
```

### 🚀 Versão 2.0 - Arquitetura Proposta

```rust
// Cache otimizado com todas melhorias
pub struct QuickleafV2 {
    // Dados principais com ordem preservada
    data: IndexMap<Arc<str>, CacheItem>,
    
    // Índices para buscas rápidas
    prefix_trie: Trie,
    suffix_trie: Trie,
    
    // Cache de queries frequentes
    query_cache: LruCache<QueryKey, Vec<Arc<str>>>,
    
    // Pool de strings para reduzir alocações
    string_pool: StringPool,
    
    // Configurações
    capacity: usize,
    default_ttl: Option<Duration>,
}
```

### 📊 Benchmark Comparativo Esperado

```
quickleaf v1 (atual):
  insert/10000: 7.3µs
  get/10000: 51ns
  remove: 2.2µs
  list_suffix: 10µs

quickleaf v2 (otimizado):
  insert/10000: 2.1µs (-71%)
  get/10000: 45ns (-12%)
  remove: 200ns (-91%)
  list_suffix: 3µs (-70%)
```

### 🔬 Próximos Passos

1. **Criar branch `optimization`**
2. **Implementar IndexMap primeiro** (maior ROI)
3. **Adicionar micro-benchmarks** para cada otimização
4. **A/B testing** com workloads reais
5. **Profile com `perf`** para identificar hot paths

---

*Análise baseada nos benchmarks realizados em AMD Ryzen 9 7900, WSL2*

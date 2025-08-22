# Model

## Embedding Model

### Qwen3-Embbedding

从 <https://hf-mirror.com/Qwen/Qwen3-Embedding-4B-GGUF/tree/main> 下载 [`Qwen3-Embedding-4B-Q4_K_M.gguf`](https://hf-mirror.com/Qwen/Qwen3-Embedding-4B-GGUF/blob/main/Qwen3-Embedding-4B-Q4_K_M.gguf)

创建最简的 `Makefile`

```makefile
FROM /Users/yangjing/data/models/qwen3/Qwen3-Embedding-4B-Q4_K_M.gguf
```

创建 ollama 模型 `ollama create qwen3_embedding:4b -f Makefile`

使用 Ollama 原生 API 测试 embedding

```shell
curl http://localhost:11434/api/embed -d '{"model": "qwen3_embedding", "input": "您好，Qwen3"}'
```


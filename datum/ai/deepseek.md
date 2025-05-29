# Deepseek

## Deepseek 本地部署

### 1. 使用 Hugging Face Transformers

创建 Python 虚拟环境

```shell
conda create -n deepseek python=3.10 -y
conda activate deepseek
```

安装依赖

```shell
pip install torch torchvision torchaudio transformers accelerate sentencepiece
```

下载模型（可选，需申请密钥）

```
git lfs install
git clone https://huggingface.co/deepseek-ai/deekseek-llm-7b-base
```

推理代码示例

```python
from transformers import AutoTokenizer, AutoModelForCausalLM
model = AutoModelForCausalLM.from_pretrained("deekseek-llm-7b-base", device_map="auto")
tokenizer = AutoTokenizer.from_pretrained("deekseek-llm-7b-base")
inputs = tokenizer("DeepSeek is", return_tensors="pt").to("cuda")
outputs = model.generate(**inputs, max_length=50)
print(tokenizer.decode(outputs[0]))
```

### 2. 使用 vLLM 加速推理

安装 vLLM

```shell
pip install vllm
```

启动 API 服务

```shell
python -m vllm.entrypoints.api_server \
  --model deekseek-llm-7b-base \
  --tensor-parallel-size 2
```

调用 API

```shell
curl http://localhost:8000/generate \
  -d '{
    "prompt": "DeepSeek 的优势是",
    "max_tokens": 100
  }'
```

### 3. Docker 部署

使用官方 Docker 镜像

```shell
docker run -it --gpus all -p 7860:7860 \
  -v /path/to/models:/models \
  deepseekai/deepseek-llm:latest \
  --model /models/deepseek-7b \
  --quantization int8 # 支持 int8/int4 量化
```

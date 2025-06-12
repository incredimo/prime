# System Operation
Timestamp: 2025-06-12 15:43:12
Operation: execute command
Status: FAILED

Exit code: 1
Output:
```
Collecting vllm
  Using cached vllm-0.9.1.tar.gz (8.7 MB)
  Installing build dependencies: started
  Installing build dependencies: finished with status 'done'
  Getting requirements to build wheel: started
  Getting requirements to build wheel: finished with status 'done'
  Preparing metadata (pyproject.toml): started
  Preparing metadata (pyproject.toml): finished with status 'done'
Requirement already satisfied: regex in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (2024.11.6)
Requirement already satisfied: cachetools in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (5.5.2)
Requirement already satisfied: psutil in c:\users\nobody\appdata\roaming\python\python310\site-packages (from vllm) (7.0.0)
Requirement already satisfied: sentencepiece in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (0.2.0)
Requirement already satisfied: numpy in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (1.26.4)
Requirement already satisfied: requests>=2.26.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (2.32.3)
Requirement already satisfied: tqdm in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (4.67.1)
Collecting blake3 (from vllm)
  Using cached blake3-1.0.5-cp310-cp310-win_amd64.whl.metadata (4.3 kB)
Collecting py-cpuinfo (from vllm)
  Using cached py_cpuinfo-9.0.0-py3-none-any.whl.metadata (794 bytes)
Requirement already satisfied: transformers>=4.51.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (4.52.2)
Collecting huggingface-hub>=0.32.0 (from huggingface-hub[hf_xet]>=0.32.0->vllm)
  Using cached huggingface_hub-0.33.0-py3-none-any.whl.metadata (14 kB)
Requirement already satisfied: tokenizers>=0.21.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (0.21.1)
Requirement already satisfied: protobuf in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (5.29.4)
Requirement already satisfied: fastapi>=0.115.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from fastapi[standard]>=0.115.0->vllm) (0.115.9)
Requirement already satisfied: aiohttp in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (3.11.16)
Requirement already satisfied: openai>=1.52.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (1.73.0)
Requirement already satisfied: pydantic>=2.10 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (2.11.3)
Requirement already satisfied: prometheus_client>=0.18.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (0.21.1)
Requirement already satisfied: pillow in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (11.1.0)
Collecting prometheus-fastapi-instrumentator>=7.0.0 (from vllm)
  Using cached prometheus_fastapi_instrumentator-7.1.0-py3-none-any.whl.metadata (13 kB)
Requirement already satisfied: tiktoken>=0.6.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (0.9.0)
Collecting lm-format-enforcer<0.11,>=0.10.11 (from vllm)
  Using cached lm_format_enforcer-0.10.11-py3-none-any.whl.metadata (17 kB)
Collecting outlines==0.1.11 (from vllm)
  Using cached outlines-0.1.11-py3-none-any.whl.metadata (17 kB)
Collecting lark==1.2.2 (from vllm)
  Using cached lark-1.2.2-py3-none-any.whl.metadata (1.8 kB)
Requirement already satisfied: typing_extensions>=4.10 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (4.13.2)
Requirement already satisfied: filelock>=3.16.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (3.18.0)
Collecting partial-json-parser (from vllm)
  Using cached partial_json_parser-0.2.1.1.post5-py3-none-any.whl.metadata (6.1 kB)
Requirement already satisfied: pyzmq>=25.0.0 in c:\users\nobody\appdata\roaming\python\python310\site-packages (from vllm) (26.4.0)
Collecting msgspec (from vllm)
  Using cached msgspec-0.19.0-cp310-cp310-win_amd64.whl.metadata (7.1 kB)
Collecting gguf>=0.13.0 (from vllm)
  Using cached gguf-0.17.0-py3-none-any.whl.metadata (4.4 kB)
Collecting mistral_common>=1.5.4 (from mistral_common[opencv]>=1.5.4->vllm)
  Using cached mistral_common-1.6.0-py3-none-any.whl.metadata (3.2 kB)
Collecting opencv-python-headless>=4.11.0 (from vllm)
  Using cached opencv_python_headless-4.11.0.86-cp37-abi3-win_amd64.whl.metadata (20 kB)
Requirement already satisfied: pyyaml in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (6.0.2)
Requirement already satisfied: einops in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (0.8.1)
Collecting compressed-tensors==0.10.1 (from vllm)
  Using cached compressed_tensors-0.10.1-py3-none-any.whl.metadata (7.0 kB)
Collecting depyf==0.18.0 (from vllm)
  Using cached depyf-0.18.0-py3-none-any.whl.metadata (7.1 kB)
Collecting cloudpickle (from vllm)
  Using cached cloudpickle-3.1.1-py3-none-any.whl.metadata (7.1 kB)
Requirement already satisfied: watchfiles in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (1.0.5)
Requirement already satisfied: python-json-logger in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (3.3.0)
Requirement already satisfied: scipy in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (1.13.1)
Collecting ninja (from vllm)
  Using cached ninja-1.11.1.4-py3-none-win_amd64.whl.metadata (5.0 kB)
Requirement already satisfied: opentelemetry-sdk>=1.26.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (1.32.0)
Requirement already satisfied: opentelemetry-api>=1.26.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from vllm) (1.32.0)
Collecting opentelemetry-exporter-otlp>=1.26.0 (from vllm)
  Using cached opentelemetry_exporter_otlp-1.34.1-py3-none-any.whl.metadata (2.4 kB)
Collecting opentelemetry-semantic-conventions-ai>=0.4.1 (from vllm)
  Using cached opentelemetry_semantic_conventions_ai-0.4.9-py3-none-any.whl.metadata (1.1 kB)
Requirement already satisfied: torch>=1.7.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from compressed-tensors==0.10.1->vllm) (2.7.0)
Collecting astor (from depyf==0.18.0->vllm)
  Using cached astor-0.8.1-py2.py3-none-any.whl.metadata (4.2 kB)
Requirement already satisfied: dill in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages\dill-0.3.8-py3.10.egg (from depyf==0.18.0->vllm) (0.3.8)
Collecting interegular (from outlines==0.1.11->vllm)
  Using cached interegular-0.3.3-py37-none-any.whl.metadata (3.0 kB)
Requirement already satisfied: jinja2 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from outlines==0.1.11->vllm) (3.1.6)
Requirement already satisfied: nest_asyncio in c:\users\nobody\appdata\roaming\python\python310\site-packages (from outlines==0.1.11->vllm) (1.6.0)
Requirement already satisfied: diskcache in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from outlines==0.1.11->vllm) (5.6.3)
Requirement already satisfied: referencing in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from outlines==0.1.11->vllm) (0.36.2)
Requirement already satisfied: jsonschema in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from outlines==0.1.11->vllm) (4.23.0)
Collecting pycountry (from outlines==0.1.11->vllm)
  Using cached pycountry-24.6.1-py3-none-any.whl.metadata (12 kB)
Collecting airportsdata (from outlines==0.1.11->vllm)
  Using cached airportsdata-20250523-py3-none-any.whl.metadata (9.1 kB)
Collecting outlines_core==0.1.26 (from outlines==0.1.11->vllm)
  Using cached outlines_core-0.1.26-cp310-cp310-win_amd64.whl.metadata (3.9 kB)
Requirement already satisfied: packaging in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from lm-format-enforcer<0.11,>=0.10.11->vllm) (24.2)
Requirement already satisfied: starlette<0.46.0,>=0.40.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from fastapi>=0.115.0->fastapi[standard]>=0.115.0->vllm) (0.45.3)
Requirement already satisfied: annotated-types>=0.6.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from pydantic>=2.10->vllm) (0.7.0)
Requirement already satisfied: pydantic-core==2.33.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from pydantic>=2.10->vllm) (2.33.1)
Requirement already satisfied: typing-inspection>=0.4.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from pydantic>=2.10->vllm) (0.4.0)
Requirement already satisfied: anyio<5,>=3.6.2 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from starlette<0.46.0,>=0.40.0->fastapi>=0.115.0->fastapi[standard]>=0.115.0->vllm) (4.9.0)
Requirement already satisfied: exceptiongroup>=1.0.2 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from anyio<5,>=3.6.2->starlette<0.46.0,>=0.40.0->fastapi>=0.115.0->fastapi[standard]>=0.115.0->vllm) (1.2.2)
Requirement already satisfied: idna>=2.8 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from anyio<5,>=3.6.2->starlette<0.46.0,>=0.40.0->fastapi>=0.115.0->fastapi[standard]>=0.115.0->vllm) (3.10)
Requirement already satisfied: sniffio>=1.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from anyio<5,>=3.6.2->starlette<0.46.0,>=0.40.0->fastapi>=0.115.0->fastapi[standard]>=0.115.0->vllm) (1.3.1)
Collecting fastapi-cli>=0.0.5 (from fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm)
  Using cached fastapi_cli-0.0.7-py3-none-any.whl.metadata (6.2 kB)
Requirement already satisfied: httpx>=0.23.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from fastapi[standard]>=0.115.0->vllm) (0.25.2)
Collecting python-multipart>=0.0.18 (from fastapi[standard]>=0.115.0->vllm)
  Using cached python_multipart-0.0.20-py3-none-any.whl.metadata (1.8 kB)
Collecting email-validator>=2.0.0 (from fastapi[standard]>=0.115.0->vllm)
  Using cached email_validator-2.2.0-py3-none-any.whl.metadata (25 kB)
Requirement already satisfied: uvicorn>=0.12.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from uvicorn[standard]>=0.12.0; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (0.34.1)
Collecting dnspython>=2.0.0 (from email-validator>=2.0.0->fastapi[standard]>=0.115.0->vllm)
  Using cached dnspython-2.7.0-py3-none-any.whl.metadata (5.8 kB)
Requirement already satisfied: typer>=0.12.3 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (0.15.2)
Collecting rich-toolkit>=0.11.1 (from fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm)
  Using cached rich_toolkit-0.14.7-py3-none-any.whl.metadata (999 bytes)
Requirement already satisfied: certifi in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from httpx>=0.23.0->fastapi[standard]>=0.115.0->vllm) (2025.1.31)
Requirement already satisfied: httpcore==1.* in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from httpx>=0.23.0->fastapi[standard]>=0.115.0->vllm) (1.0.8)
Requirement already satisfied: h11<0.15,>=0.13 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from httpcore==1.*->httpx>=0.23.0->fastapi[standard]>=0.115.0->vllm) (0.14.0)
Requirement already satisfied: fsspec>=2023.5.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from huggingface-hub>=0.32.0->huggingface-hub[hf_xet]>=0.32.0->vllm) (2025.3.2)
Collecting hf-xet<2.0.0,>=1.1.2 (from huggingface-hub[hf_xet]>=0.32.0->vllm)
  Using cached hf_xet-1.1.3-cp37-abi3-win_amd64.whl.metadata (883 bytes)
Requirement already satisfied: MarkupSafe>=2.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from jinja2->outlines==0.1.11->vllm) (3.0.2)
Requirement already satisfied: attrs>=22.2.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from jsonschema->outlines==0.1.11->vllm) (25.3.0)
Requirement already satisfied: jsonschema-specifications>=2023.03.6 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from jsonschema->outlines==0.1.11->vllm) (2024.10.1)
Requirement already satisfied: rpds-py>=0.7.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from jsonschema->outlines==0.1.11->vllm) (0.24.0)
Requirement already satisfied: distro<2,>=1.7.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from openai>=1.52.0->vllm) (1.9.0)
Requirement already satisfied: jiter<1,>=0.4.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from openai>=1.52.0->vllm) (0.9.0)
Requirement already satisfied: deprecated>=1.2.6 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from opentelemetry-api>=1.26.0->vllm) (1.2.18)
Requirement already satisfied: importlib-metadata<8.7.0,>=6.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from opentelemetry-api>=1.26.0->vllm) (7.2.1)
Requirement already satisfied: zipp>=0.5 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from importlib-metadata<8.7.0,>=6.0->opentelemetry-api>=1.26.0->vllm) (3.21.0)
Requirement already satisfied: wrapt<2,>=1.10 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from deprecated>=1.2.6->opentelemetry-api>=1.26.0->vllm) (1.17.2)
Collecting opentelemetry-exporter-otlp-proto-grpc==1.34.1 (from opentelemetry-exporter-otlp>=1.26.0->vllm)
  Using cached opentelemetry_exporter_otlp_proto_grpc-1.34.1-py3-none-any.whl.metadata (2.4 kB)
Collecting opentelemetry-exporter-otlp-proto-http==1.34.1 (from opentelemetry-exporter-otlp>=1.26.0->vllm)
  Using cached opentelemetry_exporter_otlp_proto_http-1.34.1-py3-none-any.whl.metadata (2.3 kB)
Requirement already satisfied: googleapis-common-protos~=1.52 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from opentelemetry-exporter-otlp-proto-grpc==1.34.1->opentelemetry-exporter-otlp>=1.26.0->vllm) (1.70.0)
Requirement already satisfied: grpcio<2.0.0,>=1.63.2 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from opentelemetry-exporter-otlp-proto-grpc==1.34.1->opentelemetry-exporter-otlp>=1.26.0->vllm) (1.71.0)
Collecting opentelemetry-exporter-otlp-proto-common==1.34.1 (from opentelemetry-exporter-otlp-proto-grpc==1.34.1->opentelemetry-exporter-otlp>=1.26.0->vllm)
  Using cached opentelemetry_exporter_otlp_proto_common-1.34.1-py3-none-any.whl.metadata (1.9 kB)
Collecting opentelemetry-proto==1.34.1 (from opentelemetry-exporter-otlp-proto-grpc==1.34.1->opentelemetry-exporter-otlp>=1.26.0->vllm)
  Using cached opentelemetry_proto-1.34.1-py3-none-any.whl.metadata (2.4 kB)
Collecting opentelemetry-sdk>=1.26.0 (from vllm)
  Using cached opentelemetry_sdk-1.34.1-py3-none-any.whl.metadata (1.6 kB)
Collecting opentelemetry-api>=1.26.0 (from vllm)
  Using cached opentelemetry_api-1.34.1-py3-none-any.whl.metadata (1.5 kB)
Collecting opentelemetry-semantic-conventions==0.55b1 (from opentelemetry-sdk>=1.26.0->vllm)
  Using cached opentelemetry_semantic_conventions-0.55b1-py3-none-any.whl.metadata (2.5 kB)
Requirement already satisfied: charset-normalizer<4,>=2 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from requests>=2.26.0->vllm) (3.4.1)
Requirement already satisfied: urllib3<3,>=1.21.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from requests>=2.26.0->vllm) (2.4.0)
Requirement already satisfied: click>=8.1.7 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from rich-toolkit>=0.11.1->fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (8.1.8)
Requirement already satisfied: rich>=13.7.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from rich-toolkit>=0.11.1->fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (14.0.0)
Requirement already satisfied: colorama in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from click>=8.1.7->rich-toolkit>=0.11.1->fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (0.4.6)
Requirement already satisfied: markdown-it-py>=2.2.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from rich>=13.7.1->rich-toolkit>=0.11.1->fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (3.0.0)
Requirement already satisfied: pygments<3.0.0,>=2.13.0 in c:\users\nobody\appdata\roaming\python\python310\site-packages (from rich>=13.7.1->rich-toolkit>=0.11.1->fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (2.19.1)
Requirement already satisfied: mdurl~=0.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from markdown-it-py>=2.2.0->rich>=13.7.1->rich-toolkit>=0.11.1->fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (0.1.2)
Requirement already satisfied: sympy>=1.13.3 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from torch>=1.7.0->compressed-tensors==0.10.1->vllm) (1.13.3)
Requirement already satisfied: networkx in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from torch>=1.7.0->compressed-tensors==0.10.1->vllm) (3.2.1)
Requirement already satisfied: mpmath<1.4,>=1.1.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from sympy>=1.13.3->torch>=1.7.0->compressed-tensors==0.10.1->vllm) (1.3.0)
Requirement already satisfied: safetensors>=0.4.3 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from transformers>=4.51.1->vllm) (0.5.3)
Requirement already satisfied: shellingham>=1.3.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from typer>=0.12.3->fastapi-cli>=0.0.5->fastapi-cli[standard]>=0.0.5; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (1.5.4)
Requirement already satisfied: httptools>=0.6.3 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from uvicorn[standard]>=0.12.0; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (0.6.4)
Requirement already satisfied: python-dotenv>=0.13 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from uvicorn[standard]>=0.12.0; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (1.0.0)
Requirement already satisfied: websockets>=10.4 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from uvicorn[standard]>=0.12.0; extra == "standard"->fastapi[standard]>=0.115.0->vllm) (15.0.1)
Requirement already satisfied: aiohappyeyeballs>=2.3.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from aiohttp->vllm) (2.6.1)
Requirement already satisfied: aiosignal>=1.1.2 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from aiohttp->vllm) (1.3.2)
Requirement already satisfied: async-timeout<6.0,>=4.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from aiohttp->vllm) (4.0.3)
Requirement already satisfied: frozenlist>=1.1.1 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from aiohttp->vllm) (1.5.0)
Requirement already satisfied: multidict<7.0,>=4.5 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from aiohttp->vllm) (6.4.3)
Requirement already satisfied: propcache>=0.2.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from aiohttp->vllm) (0.3.1)
Requirement already satisfied: yarl<2.0,>=1.17.0 in c:\users\nobody\appdata\local\programs\python\python310\lib\site-packages (from aiohttp->vllm) (1.19.0)
Using cached compressed_tensors-0.10.1-py3-none-any.whl (116 kB)
Using cached depyf-0.18.0-py3-none-any.whl (38 kB)
Using cached lark-1.2.2-py3-none-any.whl (111 kB)
Using cached outlines-0.1.11-py3-none-any.whl (87 kB)
Using cached outlines_core-0.1.26-cp310-cp310-win_amd64.whl (243 kB)
Using cached lm_format_enforcer-0.10.11-py3-none-any.whl (44 kB)
Using cached email_validator-2.2.0-py3-none-any.whl (33 kB)
Using cached dnspython-2.7.0-py3-none-any.whl (313 kB)
Using cached fastapi_cli-0.0.7-py3-none-any.whl (10 kB)
Using cached gguf-0.17.0-py3-none-any.whl (95 kB)
Using cached huggingface_hub-0.33.0-py3-none-any.whl (514 kB)
Using cached hf_xet-1.1.3-cp37-abi3-win_amd64.whl (2.3 MB)
Using cached interegular-0.3.3-py37-none-any.whl (23 kB)
Using cached mistral_common-1.6.0-py3-none-any.whl (48 kB)
Using cached opencv_python_headless-4.11.0.86-cp37-abi3-win_amd64.whl (39.4 MB)
Using cached opentelemetry_exporter_otlp-1.34.1-py3-none-any.whl (7.0 kB)
Using cached opentelemetry_exporter_otlp_proto_grpc-1.34.1-py3-none-any.whl (18 kB)
Using cached opentelemetry_exporter_otlp_proto_common-1.34.1-py3-none-any.whl (18 kB)
Using cached opentelemetry_exporter_otlp_proto_http-1.34.1-py3-none-any.whl (17 kB)
Using cached opentelemetry_proto-1.34.1-py3-none-any.whl (55 kB)
Using cached opentelemetry_sdk-1.34.1-py3-none-any.whl (118 kB)
Using cached opentelemetry_api-1.34.1-py3-none-any.whl (65 kB)
Using cached opentelemetry_semantic_conventions-0.55b1-py3-none-any.whl (196 kB)
Using cached opentelemetry_semantic_conventions_ai-0.4.9-py3-none-any.whl (5.6 kB)
Using cached prometheus_fastapi_instrumentator-7.1.0-py3-none-any.whl (19 kB)
Using cached python_multipart-0.0.20-py3-none-any.whl (24 kB)
Using cached rich_toolkit-0.14.7-py3-none-any.whl (24 kB)
Using cached airportsdata-20250523-py3-none-any.whl (912 kB)
Using cached astor-0.8.1-py2.py3-none-any.whl (27 kB)
Using cached blake3-1.0.5-cp310-cp310-win_amd64.whl (222 kB)
Using cached cloudpickle-3.1.1-py3-none-any.whl (20 kB)
Using cached msgspec-0.19.0-cp310-cp310-win_amd64.whl (186 kB)
Using cached ninja-1.11.1.4-py3-none-win_amd64.whl (296 kB)
Using cached partial_json_parser-0.2.1.1.post5-py3-none-any.whl (10 kB)
Using cached py_cpuinfo-9.0.0-py3-none-any.whl (22 kB)
Using cached pycountry-24.6.1-py3-none-any.whl (6.3 MB)
Building wheels for collected packages: vllm
  Building wheel for vllm (pyproject.toml): started
  Building wheel for vllm (pyproject.toml): finished with status 'error'
Failed to build vllm


STDERR:
  error: subprocess-exited-with-error
  
  Building wheel for vllm (pyproject.toml) did not run successfully.
  exit code: 1
  
  [1912 lines of output]
  C:\Users\nobody\AppData\Local\Temp\pip-build-env-69rcqgus\overlay\Lib\site-packages\torch\_subclasses\functional_tensor.py:276: UserWarning: Failed to initialize NumPy: No module named 'numpy' (Triggered internally at C:\actions-runner\_work\pytorch\pytorch\pytorch\torch\csrc\utils\tensor_numpy.cpp:81.)
    cpu = _conversion_method_template(device=torch.device("cpu"))
  vLLM only supports Linux platform (including WSL) and MacOS.Building on win32, so vLLM may not be able to run correctly
  running bdist_wheel
  running build
  running build_py
  creating build\lib\vllm
  copying vllm\beam_search.py -> build\lib\vllm
  copying vllm\collect_env.py -> build\lib\vllm
  copying vllm\config.py -> build\lib\vllm
  copying vllm\connections.py -> build\lib\vllm
  copying vllm\envs.py -> build\lib\vllm
  copying vllm\env_override.py -> build\lib\vllm
  copying vllm\forward_context.py -> build\lib\vllm
  copying vllm\jsontree.py -> build\lib\vllm
  copying vllm\logger.py -> build\lib\vllm
  copying vllm\logits_process.py -> build\lib\vllm
  copying vllm\outputs.py -> build\lib\vllm
  copying vllm\pooling_params.py -> build\lib\vllm
  copying vllm\sampling_params.py -> build\lib\vllm
  copying vllm\scalar_type.py -> build\lib\vllm
  copying vllm\scripts.py -> build\lib\vllm
  copying vllm\sequence.py -> build\lib\vllm
  copying vllm\test_utils.py -> build\lib\vllm
  copying vllm\tracing.py -> build\lib\vllm
  copying vllm\utils.py -> build\lib\vllm
  copying vllm\version.py -> build\lib\vllm
  copying vllm\_custom_ops.py -> build\lib\vllm
  copying vllm\_ipex_ops.py -> build\lib\vllm
  copying vllm\_version.py -> build\lib\vllm
  copying vllm\__init__.py -> build\lib\vllm
  creating build\lib\vllm\adapter_commons
  copying vllm\adapter_commons\layers.py -> build\lib\vllm\adapter_commons
  copying vllm\adapter_commons\models.py -> build\lib\vllm\adapter_commons
  copying vllm\adapter_commons\request.py -> build\lib\vllm\adapter_commons
  copying vllm\adapter_commons\utils.py -> build\lib\vllm\adapter_commons
  copying vllm\adapter_commons\worker_manager.py -> build\lib\vllm\adapter_commons
  copying vllm\adapter_commons\__init__.py -> build\lib\vllm\adapter_commons
  creating build\lib\vllm\assets
  copying vllm\assets\audio.py -> build\lib\vllm\assets
  copying vllm\assets\base.py -> build\lib\vllm\assets
  copying vllm\assets\image.py -> build\lib\vllm\assets
  copying vllm\assets\video.py -> build\lib\vllm\assets
  copying vllm\assets\__init__.py -> build\lib\vllm\assets
  creating build\lib\vllm\attention
  copying vllm\attention\layer.py -> build\lib\vllm\attention
  copying vllm\attention\selector.py -> build\lib\vllm\attention
  copying vllm\attention\__init__.py -> build\lib\vllm\attention
  creating build\lib\vllm\benchmarks
  copying vllm\benchmarks\datasets.py -> build\lib\vllm\benchmarks
  copying vllm\benchmarks\endpoint_request_func.py -> build\lib\vllm\benchmarks
  copying vllm\benchmarks\latency.py -> build\lib\vllm\benchmarks
  copying vllm\benchmarks\serve.py -> build\lib\vllm\benchmarks
  copying vllm\benchmarks\throughput.py -> build\lib\vllm\benchmarks
  copying vllm\benchmarks\utils.py -> build\lib\vllm\benchmarks
  copying vllm\benchmarks\__init__.py -> build\lib\vllm\benchmarks
  creating build\lib\vllm\compilation
  copying vllm\compilation\activation_quant_fusion.py -> build\lib\vllm\compilation
  copying vllm\compilation\backends.py -> build\lib\vllm\compilation
  copying vllm\compilation\base_piecewise_backend.py -> build\lib\vllm\compilation
  copying vllm\compilation\collective_fusion.py -> build\lib\vllm\compilation
  copying vllm\compilation\compiler_interface.py -> build\lib\vllm\compilation
  copying vllm\compilation\counter.py -> build\lib\vllm\compilation
  copying vllm\compilation\cuda_piecewise_backend.py -> build\lib\vllm\compilation
  copying vllm\compilation\decorators.py -> build\lib\vllm\compilation
  copying vllm\compilation\fix_functionalization.py -> build\lib\vllm\compilation
  copying vllm\compilation\fusion.py -> build\lib\vllm\compilation
  copying vllm\compilation\fx_utils.py -> build\lib\vllm\compilation
  copying vllm\compilation\inductor_pass.py -> build\lib\vllm\compilation
  copying vllm\compilation\monitor.py -> build\lib\vllm\compilation
  copying vllm\compilation\multi_output_match.py -> build\lib\vllm\compilation
  copying vllm\compilation\noop_elimination.py -> build\lib\vllm\compilation
  copying vllm\compilation\pass_manager.py -> build\lib\vllm\compilation
  copying vllm\compilation\sequence_parallelism.py -> build\lib\vllm\compilation
  copying vllm\compilation\torch25_custom_graph_pass.py -> build\lib\vllm\compilation
  copying vllm\compilation\vllm_inductor_pass.py -> build\lib\vllm\compilation
  copying vllm\compilation\wrapper.py -> build\lib\vllm\compilation
  copying vllm\compilation\__init__.py -> build\lib\vllm\compilation
  creating build\lib\vllm\core
  copying vllm\core\block_manager.py -> build\lib\vllm\core
  copying vllm\core\evictor.py -> build\lib\vllm\core
  copying vllm\core\interfaces.py -> build\lib\vllm\core
  copying vllm\core\placeholder_block_space_manager.py -> build\lib\vllm\core
  copying vllm\core\scheduler.py -> build\lib\vllm\core
  copying vllm\core\__init__.py -> build\lib\vllm\core
  creating build\lib\vllm\device_allocator
  copying vllm\device_allocator\cumem.py -> build\lib\vllm\device_allocator
  copying vllm\device_allocator\__init__.py -> build\lib\vllm\device_allocator
  creating build\lib\vllm\distributed
  copying vllm\distributed\communication_op.py -> build\lib\vllm\distributed
  copying vllm\distributed\kv_events.py -> build\lib\vllm\distributed
  copying vllm\distributed\parallel_state.py -> build\lib\vllm\distributed
  copying vllm\distributed\tpu_distributed_utils.py -> build\lib\vllm\distributed
  copying vllm\distributed\utils.py -> build\lib\vllm\distributed
  copying vllm\distributed\__init__.py -> build\lib\vllm\distributed
  creating build\lib\vllm\engine
  copying vllm\engine\arg_utils.py -> build\lib\vllm\engine
  copying vllm\engine\async_llm_engine.py -> build\lib\vllm\engine
  copying vllm\engine\async_timeout.py -> build\lib\vllm\engine
  copying vllm\engine\llm_engine.py -> build\lib\vllm\engine
  copying vllm\engine\metrics.py -> build\lib\vllm\engine
  copying vllm\engine\metrics_types.py -> build\lib\vllm\engine
  copying vllm\engine\protocol.py -> build\lib\vllm\engine
  copying vllm\engine\__init__.py -> build\lib\vllm\engine
  creating build\lib\vllm\entrypoints
  copying vllm\entrypoints\api_server.py -> build\lib\vllm\entrypoints
  copying vllm\entrypoints\chat_utils.py -> build\lib\vllm\entrypoints
  copying vllm\entrypoints\launcher.py -> build\lib\vllm\entrypoints
  copying vllm\entrypoints\llm.py -> build\lib\vllm\entrypoints
  copying vllm\entrypoints\logger.py -> build\lib\vllm\entrypoints
  copying vllm\entrypoints\score_utils.py -> build\lib\vllm\entrypoints
  copying vllm\entrypoints\ssl.py -> build\lib\vllm\entrypoints
  copying vllm\entrypoints\utils.py -> build\lib\vllm\entrypoints
  copying vllm\entrypoints\__init__.py -> build\lib\vllm\entrypoints
  creating build\lib\vllm\executor
  copying vllm\executor\executor_base.py -> build\lib\vllm\executor
  copying vllm\executor\mp_distributed_executor.py -> build\lib\vllm\executor
  copying vllm\executor\msgspec_utils.py -> build\lib\vllm\executor
  copying vllm\executor\multiproc_worker_utils.py -> build\lib\vllm\executor
  copying vllm\executor\ray_distributed_executor.py -> build\lib\vllm\executor
  copying vllm\executor\ray_utils.py -> build\lib\vllm\executor
  copying vllm\executor\uniproc_executor.py -> build\lib\vllm\executor
  copying vllm\executor\__init__.py -> build\lib\vllm\executor
  creating build\lib\vllm\inputs
  copying vllm\inputs\data.py -> build\lib\vllm\inputs
  copying vllm\inputs\parse.py -> build\lib\vllm\inputs
  copying vllm\inputs\preprocess.py -> build\lib\vllm\inputs
  copying vllm\inputs\registry.py -> build\lib\vllm\inputs
  copying vllm\inputs\__init__.py -> build\lib\vllm\inputs
  creating build\lib\vllm\logging_utils
  copying vllm\logging_utils\dump_input.py -> build\lib\vllm\logging_utils
  copying vllm\logging_utils\formatter.py -> build\lib\vllm\logging_utils
  copying vllm\logging_utils\__init__.py -> build\lib\vllm\logging_utils
  creating build\lib\vllm\lora
  copying vllm\lora\fully_sharded_layers.py -> build\lib\vllm\lora
  copying vllm\lora\layers.py -> build\lib\vllm\lora
  copying vllm\lora\lora.py -> build\lib\vllm\lora
  copying vllm\lora\models.py -> build\lib\vllm\lora
  copying vllm\lora\peft_helper.py -> build\lib\vllm\lora
  copying vllm\lora\request.py -> build\lib\vllm\lora
  copying vllm\lora\resolver.py -> build\lib\vllm\lora
  copying vllm\lora\utils.py -> build\lib\vllm\lora
  copying vllm\lora\worker_manager.py -> build\lib\vllm\lora
  copying vllm\lora\__init__.py -> build\lib\vllm\lora
  creating build\lib\vllm\model_executor
  copying vllm\model_executor\custom_op.py -> build\lib\vllm\model_executor
  copying vllm\model_executor\parameter.py -> build\lib\vllm\model_executor
  copying vllm\model_executor\pooling_metadata.py -> build\lib\vllm\model_executor
  copying vllm\model_executor\sampling_metadata.py -> build\lib\vllm\model_executor
  copying vllm\model_executor\utils.py -> build\lib\vllm\model_executor
  copying vllm\model_executor\__init__.py -> build\lib\vllm\model_executor
  creating build\lib\vllm\multimodal
  copying vllm\multimodal\audio.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\base.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\hasher.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\image.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\inputs.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\parse.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\processing.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\profiling.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\registry.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\utils.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\video.py -> build\lib\vllm\multimodal
  copying vllm\multimodal\__init__.py -> build\lib\vllm\multimodal
  creating build\lib\vllm\platforms
  copying vllm\platforms\cpu.py -> build\lib\vllm\platforms
  copying vllm\platforms\cuda.py -> build\lib\vllm\platforms
  copying vllm\platforms\hpu.py -> build\lib\vllm\platforms
  copying vllm\platforms\interface.py -> build\lib\vllm\platforms
  copying vllm\platforms\neuron.py -> build\lib\vllm\platforms
  copying vllm\platforms\rocm.py -> build\lib\vllm\platforms
  copying vllm\platforms\tpu.py -> build\lib\vllm\platforms
  copying vllm\platforms\xpu.py -> build\lib\vllm\platforms
  copying vllm\platforms\__init__.py -> build\lib\vllm\platforms
  creating build\lib\vllm\plugins
  copying vllm\plugins\__init__.py -> build\lib\vllm\plugins
  creating build\lib\vllm\profiler
  copying vllm\profiler\layerwise_profile.py -> build\lib\vllm\profiler
  copying vllm\profiler\utils.py -> build\lib\vllm\profiler
  copying vllm\profiler\__init__.py -> build\lib\vllm\profiler
  creating build\lib\vllm\prompt_adapter
  copying vllm\prompt_adapter\layers.py -> build\lib\vllm\prompt_adapter
  copying vllm\prompt_adapter\models.py -> build\lib\vllm\prompt_adapter
  copying vllm\prompt_adapter\request.py -> build\lib\vllm\prompt_adapter
  copying vllm\prompt_adapter\utils.py -> build\lib\vllm\prompt_adapter
  copying vllm\prompt_adapter\worker_manager.py -> build\lib\vllm\prompt_adapter
  copying vllm\prompt_adapter\__init__.py -> build\lib\vllm\prompt_adapter
  creating build\lib\vllm\reasoning
  copying vllm\reasoning\abs_reasoning_parsers.py -> build\lib\vllm\reasoning
  copying vllm\reasoning\deepseek_r1_reasoning_parser.py -> build\lib\vllm\reasoning
  copying vllm\reasoning\granite_reasoning_parser.py -> build\lib\vllm\reasoning
  copying vllm\reasoning\qwen3_reasoning_parser.py -> build\lib\vllm\reasoning
  copying vllm\reasoning\__init__.py -> build\lib\vllm\reasoning
  creating build\lib\vllm\spec_decode
  copying vllm\spec_decode\batch_expansion.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\draft_model_runner.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\interfaces.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\medusa_worker.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\metrics.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\mlp_speculator_worker.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\mqa_scorer.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\multi_step_worker.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\ngram_worker.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\proposer_worker_base.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\smaller_tp_proposer_worker.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\spec_decode_worker.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\target_model_runner.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\top1_proposer.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\util.py -> build\lib\vllm\spec_decode
  copying vllm\spec_decode\__init__.py -> build\lib\vllm\spec_decode
  creating build\lib\vllm\third_party
  copying vllm\third_party\pynvml.py -> build\lib\vllm\third_party
  copying vllm\third_party\__init__.py -> build\lib\vllm\third_party
  creating build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\config.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\detokenizer.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\detokenizer_utils.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\processor.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\s3_utils.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\tokenizer.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\tokenizer_base.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\tokenizer_group.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\utils.py -> build\lib\vllm\transformers_utils
  copying vllm\transformers_utils\__init__.py -> build\lib\vllm\transformers_utils
  creating build\lib\vllm\triton_utils
  copying vllm\triton_utils\importing.py -> build\lib\vllm\triton_utils
  copying vllm\triton_utils\__init__.py -> build\lib\vllm\triton_utils
  creating build\lib\vllm\usage
  copying vllm\usage\usage_lib.py -> build\lib\vllm\usage
  copying vllm\usage\__init__.py -> build\lib\vllm\usage
  creating build\lib\vllm\v1
  copying vllm\v1\kv_cache_interface.py -> build\lib\vllm\v1
  copying vllm\v1\outputs.py -> build\lib\vllm\v1
  copying vllm\v1\request.py -> build\lib\vllm\v1
  copying vllm\v1\serial_utils.py -> build\lib\vllm\v1
  copying vllm\v1\utils.py -> build\lib\vllm\v1
  copying vllm\v1\__init__.py -> build\lib\vllm\v1
  creating build\lib\vllm\vllm_flash_attn
  copying vllm\vllm_flash_attn\flash_attn_interface.py -> build\lib\vllm\vllm_flash_attn
  copying vllm\vllm_flash_attn\__init__.py -> build\lib\vllm\vllm_flash_attn
  creating build\lib\vllm\worker
  copying vllm\worker\cache_engine.py -> build\lib\vllm\worker
  copying vllm\worker\cpu_enc_dec_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\cpu_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\cpu_pooling_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\cpu_worker.py -> build\lib\vllm\worker
  copying vllm\worker\enc_dec_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\hpu_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\hpu_worker.py -> build\lib\vllm\worker
  copying vllm\worker\model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\model_runner_base.py -> build\lib\vllm\worker
  copying vllm\worker\multi_step_hpu_worker.py -> build\lib\vllm\worker
  copying vllm\worker\multi_step_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\multi_step_neuronx_distributed_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\multi_step_neuron_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\multi_step_tpu_worker.py -> build\lib\vllm\worker
  copying vllm\worker\multi_step_worker.py -> build\lib\vllm\worker
  copying vllm\worker\neuronx_distributed_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\neuron_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\neuron_worker.py -> build\lib\vllm\worker
  copying vllm\worker\pooling_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\tpu_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\tpu_worker.py -> build\lib\vllm\worker
  copying vllm\worker\utils.py -> build\lib\vllm\worker
  copying vllm\worker\worker.py -> build\lib\vllm\worker
  copying vllm\worker\worker_base.py -> build\lib\vllm\worker
  copying vllm\worker\xpu_model_runner.py -> build\lib\vllm\worker
  copying vllm\worker\xpu_worker.py -> build\lib\vllm\worker
  copying vllm\worker\__init__.py -> build\lib\vllm\worker
  creating build\lib\vllm\attention\backends
  copying vllm\attention\backends\abstract.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\blocksparse_attn.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\cpu_mla.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\dual_chunk_flash_attn.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\flashinfer.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\flashmla.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\flash_attn.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\hpu_attn.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\ipex_attn.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\pallas.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\placeholder_attn.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\rocm_aiter_mla.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\rocm_flash_attn.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\torch_sdpa.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\triton_mla.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\utils.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\xformers.py -> build\lib\vllm\attention\backends
  copying vllm\attention\backends\__init__.py -> build\lib\vllm\attention\backends
  creating build\lib\vllm\attention\ops
  copying vllm\attention\ops\chunked_prefill_paged_decode.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\flashmla.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\hpu_paged_attn.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\ipex_attn.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\merge_attn_states.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\nki_flash_attn.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\paged_attn.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\prefix_prefill.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\rocm_aiter_mla.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\rocm_aiter_paged_attn.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\triton_decode_attention.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\triton_flash_attention.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\triton_merge_attn_states.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\triton_unified_attention.py -> build\lib\vllm\attention\ops
  copying vllm\attention\ops\__init__.py -> build\lib\vllm\attention\ops
  creating build\lib\vllm\attention\utils
  copying vllm\attention\utils\fa_utils.py -> build\lib\vllm\attention\utils
  creating build\lib\vllm\attention\backends\mla
  copying vllm\attention\backends\mla\common.py -> build\lib\vllm\attention\backends\mla
  copying vllm\attention\backends\mla\__init__.py -> build\lib\vllm\attention\backends\mla
  creating build\lib\vllm\attention\ops\blocksparse_attention
  copying vllm\attention\ops\blocksparse_attention\blocksparse_attention_kernel.py -> build\lib\vllm\attention\ops\blocksparse_attention
  copying vllm\attention\ops\blocksparse_attention\interface.py -> build\lib\vllm\attention\ops\blocksparse_attention
  copying vllm\attention\ops\blocksparse_attention\utils.py -> build\lib\vllm\attention\ops\blocksparse_attention
  copying vllm\attention\ops\blocksparse_attention\__init__.py -> build\lib\vllm\attention\ops\blocksparse_attention
  creating build\lib\vllm\core\block
  copying vllm\core\block\block_table.py -> build\lib\vllm\core\block
  copying vllm\core\block\common.py -> build\lib\vllm\core\block
  copying vllm\core\block\cpu_gpu_block_allocator.py -> build\lib\vllm\core\block
  copying vllm\core\block\interfaces.py -> build\lib\vllm\core\block
  copying vllm\core\block\naive_block.py -> build\lib\vllm\core\block
  copying vllm\core\block\prefix_caching_block.py -> build\lib\vllm\core\block
  copying vllm\core\block\utils.py -> build\lib\vllm\core\block
  copying vllm\core\block\__init__.py -> build\lib\vllm\core\block
  creating build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\all2all.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\base_device_communicator.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\cpu_communicator.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\cuda_communicator.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\cuda_wrapper.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\custom_all_reduce.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\custom_all_reduce_utils.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\hpu_communicator.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\neuron_communicator.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\pynccl.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\pynccl_wrapper.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\shm_broadcast.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\tpu_communicator.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\xpu_communicator.py -> build\lib\vllm\distributed\device_communicators
  copying vllm\distributed\device_communicators\__init__.py -> build\lib\vllm\distributed\device_communicators
  creating build\lib\vllm\distributed\kv_transfer
  copying vllm\distributed\kv_transfer\kv_connector_agent.py -> build\lib\vllm\distributed\kv_transfer
  copying vllm\distributed\kv_transfer\kv_transfer_state.py -> build\lib\vllm\distributed\kv_transfer
  copying vllm\distributed\kv_transfer\__init__.py -> build\lib\vllm\distributed\kv_transfer
  creating build\lib\vllm\distributed\kv_transfer\kv_connector
  copying vllm\distributed\kv_transfer\kv_connector\base.py -> build\lib\vllm\distributed\kv_transfer\kv_connector
  copying vllm\distributed\kv_transfer\kv_connector\factory.py -> build\lib\vllm\distributed\kv_transfer\kv_connector
  copying vllm\distributed\kv_transfer\kv_connector\lmcache_connector.py -> build\lib\vllm\distributed\kv_transfer\kv_connector
  copying vllm\distributed\kv_transfer\kv_connector\mooncake_store_connector.py -> build\lib\vllm\distributed\kv_transfer\kv_connector
  copying vllm\distributed\kv_transfer\kv_connector\simple_connector.py -> build\lib\vllm\distributed\kv_transfer\kv_connector
  copying vllm\distributed\kv_transfer\kv_connector\utils.py -> build\lib\vllm\distributed\kv_transfer\kv_connector
  copying vllm\distributed\kv_transfer\kv_connector\__init__.py -> build\lib\vllm\distributed\kv_transfer\kv_connector
  creating build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer
  copying vllm\distributed\kv_transfer\kv_lookup_buffer\base.py -> build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer
  copying vllm\distributed\kv_transfer\kv_lookup_buffer\mooncake_store.py -> build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer
  copying vllm\distributed\kv_transfer\kv_lookup_buffer\simple_buffer.py -> build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer
  copying vllm\distributed\kv_transfer\kv_lookup_buffer\__init__.py -> build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer
  creating build\lib\vllm\distributed\kv_transfer\kv_pipe
  copying vllm\distributed\kv_transfer\kv_pipe\base.py -> build\lib\vllm\distributed\kv_transfer\kv_pipe
  copying vllm\distributed\kv_transfer\kv_pipe\mooncake_pipe.py -> build\lib\vllm\distributed\kv_transfer\kv_pipe
  copying vllm\distributed\kv_transfer\kv_pipe\pynccl_pipe.py -> build\lib\vllm\distributed\kv_transfer\kv_pipe
  copying vllm\distributed\kv_transfer\kv_pipe\__init__.py -> build\lib\vllm\distributed\kv_transfer\kv_pipe
  creating build\lib\vllm\distributed\kv_transfer\kv_connector\v1
  copying vllm\distributed\kv_transfer\kv_connector\v1\base.py -> build\lib\vllm\distributed\kv_transfer\kv_connector\v1
  copying vllm\distributed\kv_transfer\kv_connector\v1\lmcache_connector.py -> build\lib\vllm\distributed\kv_transfer\kv_connector\v1
  copying vllm\distributed\kv_transfer\kv_connector\v1\multi_connector.py -> build\lib\vllm\distributed\kv_transfer\kv_connector\v1
  copying vllm\distributed\kv_transfer\kv_connector\v1\nixl_connector.py -> build\lib\vllm\distributed\kv_transfer\kv_connector\v1
  copying vllm\distributed\kv_transfer\kv_connector\v1\shared_storage_connector.py -> build\lib\vllm\distributed\kv_transfer\kv_connector\v1
  copying vllm\distributed\kv_transfer\kv_connector\v1\__init__.py -> build\lib\vllm\distributed\kv_transfer\kv_connector\v1
  creating build\lib\vllm\engine\multiprocessing
  copying vllm\engine\multiprocessing\client.py -> build\lib\vllm\engine\multiprocessing
  copying vllm\engine\multiprocessing\engine.py -> build\lib\vllm\engine\multiprocessing
  copying vllm\engine\multiprocessing\__init__.py -> build\lib\vllm\engine\multiprocessing
  creating build\lib\vllm\engine\output_processor
  copying vllm\engine\output_processor\interfaces.py -> build\lib\vllm\engine\output_processor
  copying vllm\engine\output_processor\multi_step.py -> build\lib\vllm\engine\output_processor
  copying vllm\engine\output_processor\single_step.py -> build\lib\vllm\engine\output_processor
  copying vllm\engine\output_processor\stop_checker.py -> build\lib\vllm\engine\output_processor
  copying vllm\engine\output_processor\util.py -> build\lib\vllm\engine\output_processor
  copying vllm\engine\output_processor\__init__.py -> build\lib\vllm\engine\output_processor
  creating build\lib\vllm\entrypoints\cli
  copying vllm\entrypoints\cli\collect_env.py -> build\lib\vllm\entrypoints\cli
  copying vllm\entrypoints\cli\main.py -> build\lib\vllm\entrypoints\cli
  copying vllm\entrypoints\cli\openai.py -> build\lib\vllm\entrypoints\cli
  copying vllm\entrypoints\cli\run_batch.py -> build\lib\vllm\entrypoints\cli
  copying vllm\entrypoints\cli\serve.py -> build\lib\vllm\entrypoints\cli
  copying vllm\entrypoints\cli\types.py -> build\lib\vllm\entrypoints\cli
  copying vllm\entrypoints\cli\__init__.py -> build\lib\vllm\entrypoints\cli
  creating build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\api_server.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\cli_args.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\logits_processors.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\protocol.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\run_batch.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_chat.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_classification.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_completion.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_embedding.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_engine.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_models.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_pooling.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_score.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_tokenization.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\serving_transcription.py -> build\lib\vllm\entrypoints\openai
  copying vllm\entrypoints\openai\__init__.py -> build\lib\vllm\entrypoints\openai
  creating build\lib\vllm\entrypoints\cli\benchmark
  copying vllm\entrypoints\cli\benchmark\base.py -> build\lib\vllm\entrypoints\cli\benchmark
  copying vllm\entrypoints\cli\benchmark\latency.py -> build\lib\vllm\entrypoints\cli\benchmark
  copying vllm\entrypoints\cli\benchmark\main.py -> build\lib\vllm\entrypoints\cli\benchmark
  copying vllm\entrypoints\cli\benchmark\serve.py -> build\lib\vllm\entrypoints\cli\benchmark
  copying vllm\entrypoints\cli\benchmark\throughput.py -> build\lib\vllm\entrypoints\cli\benchmark
  copying vllm\entrypoints\cli\benchmark\__init__.py -> build\lib\vllm\entrypoints\cli\benchmark
  creating build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\abstract_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\deepseekv3_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\granite_20b_fc_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\granite_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\hermes_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\internlm2_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\jamba_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\llama4_pythonic_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\llama_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\mistral_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\phi4mini_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\pythonic_tool_parser.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\utils.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  copying vllm\entrypoints\openai\tool_parsers\__init__.py -> build\lib\vllm\entrypoints\openai\tool_parsers
  creating build\lib\vllm\lora\ops
  copying vllm\lora\ops\__init__.py -> build\lib\vllm\lora\ops
  creating build\lib\vllm\lora\punica_wrapper
  copying vllm\lora\punica_wrapper\punica_base.py -> build\lib\vllm\lora\punica_wrapper
  copying vllm\lora\punica_wrapper\punica_cpu.py -> build\lib\vllm\lora\punica_wrapper
  copying vllm\lora\punica_wrapper\punica_gpu.py -> build\lib\vllm\lora\punica_wrapper
  copying vllm\lora\punica_wrapper\punica_hpu.py -> build\lib\vllm\lora\punica_wrapper
  copying vllm\lora\punica_wrapper\punica_selector.py -> build\lib\vllm\lora\punica_wrapper
  copying vllm\lora\punica_wrapper\punica_tpu.py -> build\lib\vllm\lora\punica_wrapper
  copying vllm\lora\punica_wrapper\utils.py -> build\lib\vllm\lora\punica_wrapper
  copying vllm\lora\punica_wrapper\__init__.py -> build\lib\vllm\lora\punica_wrapper
  creating build\lib\vllm\lora\ops\torch_ops
  copying vllm\lora\ops\torch_ops\lora_ops.py -> build\lib\vllm\lora\ops\torch_ops
  copying vllm\lora\ops\torch_ops\__init__.py -> build\lib\vllm\lora\ops\torch_ops
  creating build\lib\vllm\lora\ops\triton_ops
  copying vllm\lora\ops\triton_ops\kernel_utils.py -> build\lib\vllm\lora\ops\triton_ops
  copying vllm\lora\ops\triton_ops\lora_expand_op.py -> build\lib\vllm\lora\ops\triton_ops
  copying vllm\lora\ops\triton_ops\lora_kernel_metadata.py -> build\lib\vllm\lora\ops\triton_ops
  copying vllm\lora\ops\triton_ops\lora_shrink_op.py -> build\lib\vllm\lora\ops\triton_ops
  copying vllm\lora\ops\triton_ops\utils.py -> build\lib\vllm\lora\ops\triton_ops
  copying vllm\lora\ops\triton_ops\__init__.py -> build\lib\vllm\lora\ops\triton_ops
  creating build\lib\vllm\lora\ops\xla_ops
  copying vllm\lora\ops\xla_ops\lora_ops.py -> build\lib\vllm\lora\ops\xla_ops
  copying vllm\lora\ops\xla_ops\__init__.py -> build\lib\vllm\lora\ops\xla_ops
  creating build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\guidance_decoding.py -> build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\guidance_logits_processors.py -> build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\guided_fields.py -> build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\lm_format_enforcer_decoding.py -> build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\outlines_decoding.py -> build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\outlines_logits_processors.py -> build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\utils.py -> build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\xgrammar_decoding.py -> build\lib\vllm\model_executor\guided_decoding
  copying vllm\model_executor\guided_decoding\__init__.py -> build\lib\vllm\model_executor\guided_decoding
  creating build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\activation.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\layernorm.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\lightning_attn.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\linear.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\logits_processor.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\pooler.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\rejection_sampler.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\resampler.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\rotary_embedding.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\sampler.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\spec_decode_base_sampler.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\typical_acceptance_sampler.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\utils.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\vocab_parallel_embedding.py -> build\lib\vllm\model_executor\layers
  copying vllm\model_executor\layers\__init__.py -> build\lib\vllm\model_executor\layers
  creating build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\adapters.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\aimv2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\arctic.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\aria.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\aya_vision.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\baichuan.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\bamba.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\bart.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\bert.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\bert_with_rope.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\blip.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\blip2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\bloom.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\chameleon.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\chatglm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\clip.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\commandr.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\constant_size_cache.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\dbrx.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\deepseek.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\deepseek_mtp.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\deepseek_v2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\deepseek_vl2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\eagle.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\exaone.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\fairseq2_llama.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\falcon.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\falcon_h1.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\florence2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\fuyu.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gemma.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gemma2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gemma3.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gemma3_mm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\glm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\glm4.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\glm4v.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gpt2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gpt_bigcode.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gpt_j.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gpt_neox.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\granite.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\granitemoe.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\granitemoehybrid.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\granitemoeshared.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\granite_speech.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\gritlm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\grok1.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\h2ovl.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\idefics2_vision_model.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\idefics3.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\interfaces.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\interfaces_base.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\internlm2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\internlm2_ve.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\internvl.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\intern_vit.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\jais.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\jamba.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\kimi_vl.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\llama.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\llama4.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\llama_eagle.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\llama_eagle3.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\llava.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\llava_next.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\llava_next_video.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\llava_onevision.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mamba.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mamba2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mamba_cache.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\medusa.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mimo.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mimo_mtp.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\minicpm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\minicpm3.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\minicpmo.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\minicpmv.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\minicpm_eagle.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\minimax_cache.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\minimax_text_01.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\minimax_vl_01.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mistral3.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mixtral.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mixtral_quant.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mllama.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mllama4.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mlp_speculator.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\modernbert.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\module_mapping.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\molmo.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\moonvit.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\mpt.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\nemotron.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\nemotron_h.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\nemotron_nas.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\nvlm_d.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\olmo.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\olmo2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\olmoe.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\opt.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\orion.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\ovis.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\paligemma.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\persimmon.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\phi.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\phi3.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\phi3v.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\phi3_small.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\phi4mm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\phi4mm_audio.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\phi4mm_utils.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\phimoe.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\pixtral.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\plamo2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\prithvi_geospatial_mae.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen2_5_omni_thinker.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen2_5_vl.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen2_audio.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen2_moe.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen2_rm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen2_vl.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen3.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen3_moe.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\qwen_vl.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\registry.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\roberta.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\siglip.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\skyworkr1v.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\smolvlm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\solar.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\stablelm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\starcoder2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\tarsier.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\telechat2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\teleflm.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\transformers.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\ultravox.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\utils.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\vision.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\whisper.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\zamba2.py -> build\lib\vllm\model_executor\models
  copying vllm\model_executor\models\__init__.py -> build\lib\vllm\model_executor\models
  creating build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\base_loader.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\bitsandbytes_loader.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\default_loader.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\dummy_loader.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\gguf_loader.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\neuron.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\neuronx_distributed.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\runai_streamer_loader.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\sharded_state_loader.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\tensorizer.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\tensorizer_loader.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\tpu.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\utils.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\weight_utils.py -> build\lib\vllm\model_executor\model_loader
  copying vllm\model_executor\model_loader\__init__.py -> build\lib\vllm\model_executor\model_loader
  creating build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\batched_deep_gemm_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\batched_triton_or_deep_gemm_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\cutlass_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\deepep_ht_prepare_finalize.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\deepep_ll_prepare_finalize.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\deep_gemm_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\fused_batched_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\fused_marlin_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\fused_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\layer.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\modular_kernel.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\moe_align_block_size.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\moe_pallas.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\moe_permute_unpermute.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\moe_torch_iterative.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\pplx_prepare_finalize.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\prepare_finalize.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\rocm_aiter_fused_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\triton_deep_gemm_moe.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\utils.py -> build\lib\vllm\model_executor\layers\fused_moe
  copying vllm\model_executor\layers\fused_moe\__init__.py -> build\lib\vllm\model_executor\layers\fused_moe
  creating build\lib\vllm\model_executor\layers\mamba
  copying vllm\model_executor\layers\mamba\mamba2_metadata.py -> build\lib\vllm\model_executor\layers\mamba
  copying vllm\model_executor\layers\mamba\mamba_mixer.py -> build\lib\vllm\model_executor\layers\mamba
  copying vllm\model_executor\layers\mamba\mamba_mixer2.py -> build\lib\vllm\model_executor\layers\mamba
  copying vllm\model_executor\layers\mamba\__init__.py -> build\lib\vllm\model_executor\layers\mamba
  creating build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\aqlm.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\auto_round.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\awq.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\awq_marlin.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\awq_triton.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\base_config.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\bitblas.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\bitsandbytes.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\deepspeedfp.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\experts_int8.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\fbgemm_fp8.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\fp8.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\gguf.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\gptq.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\gptq_bitblas.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\gptq_marlin.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\gptq_marlin_24.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\hqq_marlin.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\ipex_quant.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\kv_cache.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\marlin.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\modelopt.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\moe_wna16.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\neuron_quant.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\ptpc_fp8.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\qqq.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\schema.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\torchao.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\tpu_int8.py -> build\lib\vllm\model_executor\layers\quantization
  copying vllm\model_executor\layers\quantization\__init__.py -> build\lib\vllm\model_executor\layers\quantization
  creating build\lib\vllm\model_executor\layers\mamba\ops
  copying vllm\model_executor\layers\mamba\ops\causal_conv1d.py -> build\lib\vllm\model_executor\layers\mamba\ops
  copying vllm\model_executor\layers\mamba\ops\mamba_ssm.py -> build\lib\vllm\model_executor\layers\mamba\ops
  copying vllm\model_executor\layers\mamba\ops\ssd_bmm.py -> build\lib\vllm\model_executor\layers\mamba\ops
  copying vllm\model_executor\layers\mamba\ops\ssd_chunk_scan.py -> build\lib\vllm\model_executor\layers\mamba\ops
  copying vllm\model_executor\layers\mamba\ops\ssd_chunk_state.py -> build\lib\vllm\model_executor\layers\mamba\ops
  copying vllm\model_executor\layers\mamba\ops\ssd_combined.py -> build\lib\vllm\model_executor\layers\mamba\ops
  copying vllm\model_executor\layers\mamba\ops\ssd_state_passing.py -> build\lib\vllm\model_executor\layers\mamba\ops
  copying vllm\model_executor\layers\mamba\ops\__init__.py -> build\lib\vllm\model_executor\layers\mamba\ops
  creating build\lib\vllm\model_executor\layers\quantization\compressed_tensors
  copying vllm\model_executor\layers\quantization\compressed_tensors\compressed_tensors.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors
  copying vllm\model_executor\layers\quantization\compressed_tensors\compressed_tensors_moe.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors
  copying vllm\model_executor\layers\quantization\compressed_tensors\triton_scaled_mm.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors
  copying vllm\model_executor\layers\quantization\compressed_tensors\utils.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors
  copying vllm\model_executor\layers\quantization\compressed_tensors\__init__.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors
  creating build\lib\vllm\model_executor\layers\quantization\kernels
  copying vllm\model_executor\layers\quantization\kernels\__init__.py -> build\lib\vllm\model_executor\layers\quantization\kernels
  creating build\lib\vllm\model_executor\layers\quantization\quark
  copying vllm\model_executor\layers\quantization\quark\quark.py -> build\lib\vllm\model_executor\layers\quantization\quark
  copying vllm\model_executor\layers\quantization\quark\quark_moe.py -> build\lib\vllm\model_executor\layers\quantization\quark
  copying vllm\model_executor\layers\quantization\quark\utils.py -> build\lib\vllm\model_executor\layers\quantization\quark
  copying vllm\model_executor\layers\quantization\quark\__init__.py -> build\lib\vllm\model_executor\layers\quantization\quark
  creating build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\allspark_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\bitblas_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\fp8_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\gptq_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\int8_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\layer_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\machete_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\marlin_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\marlin_utils_fp4.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\marlin_utils_fp8.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\marlin_utils_test.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\marlin_utils_test_24.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\marlin_utils_test_qqq.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\mxfp4_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\nvfp4_emulation_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\quant_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\w8a8_utils.py -> build\lib\vllm\model_executor\layers\quantization\utils
  copying vllm\model_executor\layers\quantization\utils\__init__.py -> build\lib\vllm\model_executor\layers\quantization\utils
  creating build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_24.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_scheme.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w4a16_24.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w4a16_nvfp4.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w4a4_nvfp4.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w8a16_fp8.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w8a8_fp8.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w8a8_int8.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_wNa16.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying vllm\model_executor\layers\quantization\compressed_tensors\schemes\__init__.py -> build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  creating build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying vllm\model_executor\layers\quantization\kernels\mixed_precision\allspark.py -> build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying vllm\model_executor\layers\quantization\kernels\mixed_precision\bitblas.py -> build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying vllm\model_executor\layers\quantization\kernels\mixed_precision\exllama.py -> build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying vllm\model_executor\layers\quantization\kernels\mixed_precision\machete.py -> build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying vllm\model_executor\layers\quantization\kernels\mixed_precision\marlin.py -> build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying vllm\model_executor\layers\quantization\kernels\mixed_precision\MPLinearKernel.py -> build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying vllm\model_executor\layers\quantization\kernels\mixed_precision\__init__.py -> build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision
  creating build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying vllm\model_executor\layers\quantization\kernels\scaled_mm\aiter.py -> build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying vllm\model_executor\layers\quantization\kernels\scaled_mm\cutlass.py -> build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying vllm\model_executor\layers\quantization\kernels\scaled_mm\ScaledMMLinearKernel.py -> build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying vllm\model_executor\layers\quantization\kernels\scaled_mm\triton.py -> build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying vllm\model_executor\layers\quantization\kernels\scaled_mm\xla.py -> build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying vllm\model_executor\layers\quantization\kernels\scaled_mm\__init__.py -> build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm
  creating build\lib\vllm\model_executor\layers\quantization\quark\schemes
  copying vllm\model_executor\layers\quantization\quark\schemes\quark_scheme.py -> build\lib\vllm\model_executor\layers\quantization\quark\schemes
  copying vllm\model_executor\layers\quantization\quark\schemes\quark_w4a4_mxfp4.py -> build\lib\vllm\model_executor\layers\quantization\quark\schemes
  copying vllm\model_executor\layers\quantization\quark\schemes\quark_w8a8_fp8.py -> build\lib\vllm\model_executor\layers\quantization\quark\schemes
  copying vllm\model_executor\layers\quantization\quark\schemes\quark_w8a8_int8.py -> build\lib\vllm\model_executor\layers\quantization\quark\schemes
  copying vllm\model_executor\layers\quantization\quark\schemes\__init__.py -> build\lib\vllm\model_executor\layers\quantization\quark\schemes
  creating build\lib\vllm\plugins\lora_resolvers
  copying vllm\plugins\lora_resolvers\filesystem_resolver.py -> build\lib\vllm\plugins\lora_resolvers
  copying vllm\plugins\lora_resolvers\__init__.py -> build\lib\vllm\plugins\lora_resolvers
  creating build\lib\vllm\transformers_utils\chat_templates
  copying vllm\transformers_utils\chat_templates\registry.py -> build\lib\vllm\transformers_utils\chat_templates
  copying vllm\transformers_utils\chat_templates\__init__.py -> build\lib\vllm\transformers_utils\chat_templates
  creating build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\arctic.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\chatglm.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\cohere2.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\dbrx.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\deepseek_vl2.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\eagle.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\exaone.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\falcon.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\h2ovl.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\internvl.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\jais.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\kimi_vl.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\medusa.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\minimax_text_01.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\minimax_vl_01.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\mllama.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\mlp_speculator.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\moonvit.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\mpt.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\nemotron.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\nemotron_h.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\nvlm_d.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\ovis.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\skyworkr1v.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\solar.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\telechat2.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\ultravox.py -> build\lib\vllm\transformers_utils\configs
  copying vllm\transformers_utils\configs\__init__.py -> build\lib\vllm\transformers_utils\configs
  creating build\lib\vllm\transformers_utils\processors
  copying vllm\transformers_utils\processors\deepseek_vl2.py -> build\lib\vllm\transformers_utils\processors
  copying vllm\transformers_utils\processors\ovis.py -> build\lib\vllm\transformers_utils\processors
  copying vllm\transformers_utils\processors\__init__.py -> build\lib\vllm\transformers_utils\processors
  creating build\lib\vllm\transformers_utils\tokenizers
  copying vllm\transformers_utils\tokenizers\mistral.py -> build\lib\vllm\transformers_utils\tokenizers
  copying vllm\transformers_utils\tokenizers\__init__.py -> build\lib\vllm\transformers_utils\tokenizers
  creating build\lib\vllm\v1\attention
  copying vllm\v1\attention\__init__.py -> build\lib\vllm\v1\attention
  creating build\lib\vllm\v1\core
  copying vllm\v1\core\block_pool.py -> build\lib\vllm\v1\core
  copying vllm\v1\core\encoder_cache_manager.py -> build\lib\vllm\v1\core
  copying vllm\v1\core\kv_cache_coordinator.py -> build\lib\vllm\v1\core
  copying vllm\v1\core\kv_cache_manager.py -> build\lib\vllm\v1\core
  copying vllm\v1\core\kv_cache_utils.py -> build\lib\vllm\v1\core
  copying vllm\v1\core\single_type_kv_cache_manager.py -> build\lib\vllm\v1\core
  copying vllm\v1\core\__init__.py -> build\lib\vllm\v1\core
  creating build\lib\vllm\v1\engine
  copying vllm\v1\engine\async_llm.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\coordinator.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\core.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\core_client.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\detokenizer.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\exceptions.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\llm_engine.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\logprobs.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\mm_input_cache.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\output_processor.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\parallel_sampling.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\processor.py -> build\lib\vllm\v1\engine
  copying vllm\v1\engine\__init__.py -> build\lib\vllm\v1\engine
  creating build\lib\vllm\v1\executor
  copying vllm\v1\executor\abstract.py -> build\lib\vllm\v1\executor
  copying vllm\v1\executor\multiproc_executor.py -> build\lib\vllm\v1\executor
  copying vllm\v1\executor\ray_distributed_executor.py -> build\lib\vllm\v1\executor
  copying vllm\v1\executor\__init__.py -> build\lib\vllm\v1\executor
  creating build\lib\vllm\v1\metrics
  copying vllm\v1\metrics\loggers.py -> build\lib\vllm\v1\metrics
  copying vllm\v1\metrics\prometheus.py -> build\lib\vllm\v1\metrics
  copying vllm\v1\metrics\ray_wrappers.py -> build\lib\vllm\v1\metrics
  copying vllm\v1\metrics\reader.py -> build\lib\vllm\v1\metrics
  copying vllm\v1\metrics\stats.py -> build\lib\vllm\v1\metrics
  copying vllm\v1\metrics\__init__.py -> build\lib\vllm\v1\metrics
  creating build\lib\vllm\v1\sample
  copying vllm\v1\sample\metadata.py -> build\lib\vllm\v1\sample
  copying vllm\v1\sample\rejection_sampler.py -> build\lib\vllm\v1\sample
  copying vllm\v1\sample\sampler.py -> build\lib\vllm\v1\sample
  copying vllm\v1\sample\__init__.py -> build\lib\vllm\v1\sample
  creating build\lib\vllm\v1\spec_decode
  copying vllm\v1\spec_decode\eagle.py -> build\lib\vllm\v1\spec_decode
  copying vllm\v1\spec_decode\medusa.py -> build\lib\vllm\v1\spec_decode
  copying vllm\v1\spec_decode\metadata.py -> build\lib\vllm\v1\spec_decode
  copying vllm\v1\spec_decode\metrics.py -> build\lib\vllm\v1\spec_decode
  copying vllm\v1\spec_decode\ngram_proposer.py -> build\lib\vllm\v1\spec_decode
  copying vllm\v1\spec_decode\utils.py -> build\lib\vllm\v1\spec_decode
  copying vllm\v1\spec_decode\__init__.py -> build\lib\vllm\v1\spec_decode
  creating build\lib\vllm\v1\structured_output
  copying vllm\v1\structured_output\backend_guidance.py -> build\lib\vllm\v1\structured_output
  copying vllm\v1\structured_output\backend_types.py -> build\lib\vllm\v1\structured_output
  copying vllm\v1\structured_output\backend_xgrammar.py -> build\lib\vllm\v1\structured_output
  copying vllm\v1\structured_output\request.py -> build\lib\vllm\v1\structured_output
  copying vllm\v1\structured_output\utils.py -> build\lib\vllm\v1\structured_output
  copying vllm\v1\structured_output\__init__.py -> build\lib\vllm\v1\structured_output
  creating build\lib\vllm\v1\worker
  copying vllm\v1\worker\block_table.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\cpu_model_runner.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\cpu_worker.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\gpu_input_batch.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\gpu_model_runner.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\gpu_worker.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\lora_model_runner_mixin.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\tpu_model_runner.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\tpu_worker.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\utils.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\worker_base.py -> build\lib\vllm\v1\worker
  copying vllm\v1\worker\__init__.py -> build\lib\vllm\v1\worker
  creating build\lib\vllm\v1\attention\backends
  copying vllm\v1\attention\backends\cpu_attn.py -> build\lib\vllm\v1\attention\backends
  copying vllm\v1\attention\backends\flashinfer.py -> build\lib\vllm\v1\attention\backends
  copying vllm\v1\attention\backends\flash_attn.py -> build\lib\vllm\v1\attention\backends
  copying vllm\v1\attention\backends\flex_attention.py -> build\lib\vllm\v1\attention\backends
  copying vllm\v1\attention\backends\pallas.py -> build\lib\vllm\v1\attention\backends
  copying vllm\v1\attention\backends\triton_attn.py -> build\lib\vllm\v1\attention\backends
  copying vllm\v1\attention\backends\utils.py -> build\lib\vllm\v1\attention\backends
  copying vllm\v1\attention\backends\__init__.py -> build\lib\vllm\v1\attention\backends
  creating build\lib\vllm\v1\attention\backends\mla
  copying vllm\v1\attention\backends\mla\common.py -> build\lib\vllm\v1\attention\backends\mla
  copying vllm\v1\attention\backends\mla\cutlass_mla.py -> build\lib\vllm\v1\attention\backends\mla
  copying vllm\v1\attention\backends\mla\flashmla.py -> build\lib\vllm\v1\attention\backends\mla
  copying vllm\v1\attention\backends\mla\rocm_aiter_mla.py -> build\lib\vllm\v1\attention\backends\mla
  copying vllm\v1\attention\backends\mla\triton_mla.py -> build\lib\vllm\v1\attention\backends\mla
  copying vllm\v1\attention\backends\mla\__init__.py -> build\lib\vllm\v1\attention\backends\mla
  creating build\lib\vllm\v1\core\sched
  copying vllm\v1\core\sched\interface.py -> build\lib\vllm\v1\core\sched
  copying vllm\v1\core\sched\output.py -> build\lib\vllm\v1\core\sched
  copying vllm\v1\core\sched\scheduler.py -> build\lib\vllm\v1\core\sched
  copying vllm\v1\core\sched\utils.py -> build\lib\vllm\v1\core\sched
  copying vllm\v1\core\sched\__init__.py -> build\lib\vllm\v1\core\sched
  creating build\lib\vllm\v1\sample\ops
  copying vllm\v1\sample\ops\bad_words.py -> build\lib\vllm\v1\sample\ops
  copying vllm\v1\sample\ops\penalties.py -> build\lib\vllm\v1\sample\ops
  copying vllm\v1\sample\ops\topk_topp_sampler.py -> build\lib\vllm\v1\sample\ops
  copying vllm\v1\sample\ops\__init__.py -> build\lib\vllm\v1\sample\ops
  creating build\lib\vllm\v1\sample\tpu
  copying vllm\v1\sample\tpu\metadata.py -> build\lib\vllm\v1\sample\tpu
  copying vllm\v1\sample\tpu\sampler.py -> build\lib\vllm\v1\sample\tpu
  copying vllm\v1\sample\tpu\__init__.py -> build\lib\vllm\v1\sample\tpu
  running egg_info
  writing vllm.egg-info\PKG-INFO
  writing dependency_links to vllm.egg-info\dependency_links.txt
  writing entry points to vllm.egg-info\entry_points.txt
  writing requirements to vllm.egg-info\requires.txt
  writing top-level names to vllm.egg-info\top_level.txt
  ERROR setuptools_scm._file_finders.git listing git files failed - pretending there aren't any
  reading manifest file 'vllm.egg-info\SOURCES.txt'
  reading manifest template 'MANIFEST.in'
  adding license file 'LICENSE'
  writing manifest file 'vllm.egg-info\SOURCES.txt'
  copying vllm\py.typed -> build\lib\vllm
  creating build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=14336,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=14336,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=1792,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=1792,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=3072,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=3072,device_name=NVIDIA_H100_80GB_HBM3,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=3072,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=3584,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=3584,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=1,N=7168,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=1024,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=1024,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_H20-3e.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_H20.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H20-3e.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H20.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=512,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=NVIDIA_H20.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=128,N=96,device_name=NVIDIA_H20.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=1024,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=1024,device_name=NVIDIA_H100.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=1344,device_name=NVIDIA_A100-SXM4-40GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=1344,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=1344,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=14336,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=14336,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=1792,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=1792,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=2688,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=2688,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=3072,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=3072,device_name=NVIDIA_H100_80GB_HBM3,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=3200,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=3584,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=3584,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=6400,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=7168,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=int8_w8a16.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=16,N=800,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=160,N=192,device_name=NVIDIA_A800-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=1024,device_name=AMD_Instinct_MI325X,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=1024,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_H20-3e,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=512,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=256,N=64,device_name=NVIDIA_A800-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=60,N=1408,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=60,N=176,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=60,N=352,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=60,N=704,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_A800-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=2560,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=2560,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=2560,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=320,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=320,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=320,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=320,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_A800-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_GeForce_RTX_4090,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=64,N=896,device_name=NVIDIA_H20.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=AMD_Instinct_MI325X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=16384,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=16384,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=16384,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=16384,device_name=AMD_Instinct_MI325X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=AMD_Instinct_MI325X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_A100-SXM4-40GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=AMD_Instinct_MI325X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=AMD_Instinct_MI325X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_A100-SXM4-40GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_GeForce_RTX_4090,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_L40S.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=AMD_Instinct_MI325X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=AMD_Instinct_MI325X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_A100-SXM4-80GB.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_H100_80GB_HBM3.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_H200.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=AMD_Instinct_MI300X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=AMD_Instinct_MI325X.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\lib\vllm\model_executor\layers\fused_moe\configs
  creating build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=1536,K=7168,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2048,K=512,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=2304,K=7168,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=24576,K=7168,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=256,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=256,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=256,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=256,K=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=256,K=7168,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=256,K=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=256,K=7168,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=256,K=7168,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=1536,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=1536,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=1536,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=1536,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=1536,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=1536,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=1536,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=7168,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=7168,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=3072,K=7168,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=32768,K=512,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=36864,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=36864,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=36864,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=36864,K=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=36864,K=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4096,K=512,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4096,K=512,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4096,K=512,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4096,K=512,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4096,K=512,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4096,K=512,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4096,K=512,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4096,K=512,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4608,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4608,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4608,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4608,K=7168,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4608,K=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4608,K=7168,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4608,K=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=4608,K=7168,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=512,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=512,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=512,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=512,K=7168,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=512,K=7168,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=512,K=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=512,K=7168,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=576,K=7168,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1024,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=1152,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=128,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=128,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=128,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=128,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=128,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=128,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=128,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=128,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=16384,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=18432,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2048,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2048,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2048,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2048,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2048,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2048,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2048,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2048,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2304,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2304,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2304,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2304,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2304,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2304,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2304,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=2304,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=256,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=256,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=256,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=256,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=256,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=256,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=256,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=8192,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=8192,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=7168,K=8192,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=8192,K=1536,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=8192,K=1536,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\model_executor\layers\quantization\utils\configs\N=8192,K=1536,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\lib\vllm\model_executor\layers\quantization\utils\configs
  copying vllm\vllm_flash_attn\.gitkeep -> build\lib\vllm\vllm_flash_attn
  copying vllm\distributed\kv_transfer\README.md -> build\lib\vllm\distributed\kv_transfer
  copying vllm\distributed\kv_transfer\disagg_prefill_workflow.jpg -> build\lib\vllm\distributed\kv_transfer
  copying vllm\model_executor\layers\fused_moe\configs\README -> build\lib\vllm\model_executor\layers\fused_moe\configs
  copying vllm\plugins\lora_resolvers\README.md -> build\lib\vllm\plugins\lora_resolvers
  copying vllm\transformers_utils\chat_templates\template_basic.jinja -> build\lib\vllm\transformers_utils\chat_templates
  copying vllm\transformers_utils\chat_templates\template_blip2.jinja -> build\lib\vllm\transformers_utils\chat_templates
  copying vllm\transformers_utils\chat_templates\template_chatml.jinja -> build\lib\vllm\transformers_utils\chat_templates
  copying vllm\transformers_utils\chat_templates\template_deepseek_vl2.jinja -> build\lib\vllm\transformers_utils\chat_templates
  copying vllm\transformers_utils\chat_templates\template_fuyu.jinja -> build\lib\vllm\transformers_utils\chat_templates
  installing to build\bdist.win-amd64\wheel
  running install
  running install_lib
  creating build\bdist.win-amd64\wheel
  creating build\bdist.win-amd64\wheel\vllm
  creating build\bdist.win-amd64\wheel\vllm\adapter_commons
  copying build\lib\vllm\adapter_commons\layers.py -> build\bdist.win-amd64\wheel\.\vllm\adapter_commons
  copying build\lib\vllm\adapter_commons\models.py -> build\bdist.win-amd64\wheel\.\vllm\adapter_commons
  copying build\lib\vllm\adapter_commons\request.py -> build\bdist.win-amd64\wheel\.\vllm\adapter_commons
  copying build\lib\vllm\adapter_commons\utils.py -> build\bdist.win-amd64\wheel\.\vllm\adapter_commons
  copying build\lib\vllm\adapter_commons\worker_manager.py -> build\bdist.win-amd64\wheel\.\vllm\adapter_commons
  copying build\lib\vllm\adapter_commons\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\adapter_commons
  creating build\bdist.win-amd64\wheel\vllm\assets
  copying build\lib\vllm\assets\audio.py -> build\bdist.win-amd64\wheel\.\vllm\assets
  copying build\lib\vllm\assets\base.py -> build\bdist.win-amd64\wheel\.\vllm\assets
  copying build\lib\vllm\assets\image.py -> build\bdist.win-amd64\wheel\.\vllm\assets
  copying build\lib\vllm\assets\video.py -> build\bdist.win-amd64\wheel\.\vllm\assets
  copying build\lib\vllm\assets\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\assets
  creating build\bdist.win-amd64\wheel\vllm\attention
  creating build\bdist.win-amd64\wheel\vllm\attention\backends
  copying build\lib\vllm\attention\backends\abstract.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\blocksparse_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\cpu_mla.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\dual_chunk_flash_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\flashinfer.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\flashmla.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\flash_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\hpu_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\ipex_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  creating build\bdist.win-amd64\wheel\vllm\attention\backends\mla
  copying build\lib\vllm\attention\backends\mla\common.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends\mla
  copying build\lib\vllm\attention\backends\mla\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends\mla
  copying build\lib\vllm\attention\backends\pallas.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\placeholder_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\rocm_aiter_mla.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\rocm_flash_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\torch_sdpa.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\triton_mla.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\utils.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\xformers.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\backends\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\attention\backends
  copying build\lib\vllm\attention\layer.py -> build\bdist.win-amd64\wheel\.\vllm\attention
  creating build\bdist.win-amd64\wheel\vllm\attention\ops
  creating build\bdist.win-amd64\wheel\vllm\attention\ops\blocksparse_attention
  copying build\lib\vllm\attention\ops\blocksparse_attention\blocksparse_attention_kernel.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops\blocksparse_attention
  copying build\lib\vllm\attention\ops\blocksparse_attention\interface.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops\blocksparse_attention
  copying build\lib\vllm\attention\ops\blocksparse_attention\utils.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops\blocksparse_attention
  copying build\lib\vllm\attention\ops\blocksparse_attention\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops\blocksparse_attention
  copying build\lib\vllm\attention\ops\chunked_prefill_paged_decode.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\flashmla.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\hpu_paged_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\ipex_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\merge_attn_states.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\nki_flash_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\paged_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\prefix_prefill.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\rocm_aiter_mla.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\rocm_aiter_paged_attn.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\triton_decode_attention.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\triton_flash_attention.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\triton_merge_attn_states.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\triton_unified_attention.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\ops\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\attention\ops
  copying build\lib\vllm\attention\selector.py -> build\bdist.win-amd64\wheel\.\vllm\attention
  creating build\bdist.win-amd64\wheel\vllm\attention\utils
  copying build\lib\vllm\attention\utils\fa_utils.py -> build\bdist.win-amd64\wheel\.\vllm\attention\utils
  copying build\lib\vllm\attention\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\attention
  copying build\lib\vllm\beam_search.py -> build\bdist.win-amd64\wheel\.\vllm
  creating build\bdist.win-amd64\wheel\vllm\benchmarks
  copying build\lib\vllm\benchmarks\datasets.py -> build\bdist.win-amd64\wheel\.\vllm\benchmarks
  copying build\lib\vllm\benchmarks\endpoint_request_func.py -> build\bdist.win-amd64\wheel\.\vllm\benchmarks
  copying build\lib\vllm\benchmarks\latency.py -> build\bdist.win-amd64\wheel\.\vllm\benchmarks
  copying build\lib\vllm\benchmarks\serve.py -> build\bdist.win-amd64\wheel\.\vllm\benchmarks
  copying build\lib\vllm\benchmarks\throughput.py -> build\bdist.win-amd64\wheel\.\vllm\benchmarks
  copying build\lib\vllm\benchmarks\utils.py -> build\bdist.win-amd64\wheel\.\vllm\benchmarks
  copying build\lib\vllm\benchmarks\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\benchmarks
  copying build\lib\vllm\collect_env.py -> build\bdist.win-amd64\wheel\.\vllm
  creating build\bdist.win-amd64\wheel\vllm\compilation
  copying build\lib\vllm\compilation\activation_quant_fusion.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\backends.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\base_piecewise_backend.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\collective_fusion.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\compiler_interface.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\counter.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\cuda_piecewise_backend.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\decorators.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\fix_functionalization.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\fusion.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\fx_utils.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\inductor_pass.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\monitor.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\multi_output_match.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\noop_elimination.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\pass_manager.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\sequence_parallelism.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\torch25_custom_graph_pass.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\vllm_inductor_pass.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\wrapper.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\compilation\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\compilation
  copying build\lib\vllm\config.py -> build\bdist.win-amd64\wheel\.\vllm
  copying build\lib\vllm\connections.py -> build\bdist.win-amd64\wheel\.\vllm
  creating build\bdist.win-amd64\wheel\vllm\core
  creating build\bdist.win-amd64\wheel\vllm\core\block
  copying build\lib\vllm\core\block\block_table.py -> build\bdist.win-amd64\wheel\.\vllm\core\block
  copying build\lib\vllm\core\block\common.py -> build\bdist.win-amd64\wheel\.\vllm\core\block
  copying build\lib\vllm\core\block\cpu_gpu_block_allocator.py -> build\bdist.win-amd64\wheel\.\vllm\core\block
  copying build\lib\vllm\core\block\interfaces.py -> build\bdist.win-amd64\wheel\.\vllm\core\block
  copying build\lib\vllm\core\block\naive_block.py -> build\bdist.win-amd64\wheel\.\vllm\core\block
  copying build\lib\vllm\core\block\prefix_caching_block.py -> build\bdist.win-amd64\wheel\.\vllm\core\block
  copying build\lib\vllm\core\block\utils.py -> build\bdist.win-amd64\wheel\.\vllm\core\block
  copying build\lib\vllm\core\block\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\core\block
  copying build\lib\vllm\core\block_manager.py -> build\bdist.win-amd64\wheel\.\vllm\core
  copying build\lib\vllm\core\evictor.py -> build\bdist.win-amd64\wheel\.\vllm\core
  copying build\lib\vllm\core\interfaces.py -> build\bdist.win-amd64\wheel\.\vllm\core
  copying build\lib\vllm\core\placeholder_block_space_manager.py -> build\bdist.win-amd64\wheel\.\vllm\core
  copying build\lib\vllm\core\scheduler.py -> build\bdist.win-amd64\wheel\.\vllm\core
  copying build\lib\vllm\core\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\core
  creating build\bdist.win-amd64\wheel\vllm\device_allocator
  copying build\lib\vllm\device_allocator\cumem.py -> build\bdist.win-amd64\wheel\.\vllm\device_allocator
  copying build\lib\vllm\device_allocator\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\device_allocator
  creating build\bdist.win-amd64\wheel\vllm\distributed
  copying build\lib\vllm\distributed\communication_op.py -> build\bdist.win-amd64\wheel\.\vllm\distributed
  creating build\bdist.win-amd64\wheel\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\all2all.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\base_device_communicator.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\cpu_communicator.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\cuda_communicator.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\cuda_wrapper.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\custom_all_reduce.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\custom_all_reduce_utils.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\hpu_communicator.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\neuron_communicator.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\pynccl.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\pynccl_wrapper.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\shm_broadcast.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\tpu_communicator.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\xpu_communicator.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\device_communicators\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\device_communicators
  copying build\lib\vllm\distributed\kv_events.py -> build\bdist.win-amd64\wheel\.\vllm\distributed
  creating build\bdist.win-amd64\wheel\vllm\distributed\kv_transfer
  copying build\lib\vllm\distributed\kv_transfer\disagg_prefill_workflow.jpg -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer
  creating build\bdist.win-amd64\wheel\vllm\distributed\kv_transfer\kv_connector
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\base.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\factory.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\lmcache_connector.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\mooncake_store_connector.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\simple_connector.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\utils.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector
  creating build\bdist.win-amd64\wheel\vllm\distributed\kv_transfer\kv_connector\v1
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\v1\base.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector\v1
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\v1\lmcache_connector.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector\v1
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\v1\multi_connector.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector\v1
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\v1\nixl_connector.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector\v1
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\v1\shared_storage_connector.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector\v1
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\v1\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector\v1
  copying build\lib\vllm\distributed\kv_transfer\kv_connector\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_connector
  copying build\lib\vllm\distributed\kv_transfer\kv_connector_agent.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer
  creating build\bdist.win-amd64\wheel\vllm\distributed\kv_transfer\kv_lookup_buffer
  copying build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer\base.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_lookup_buffer
  copying build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer\mooncake_store.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_lookup_buffer
  copying build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer\simple_buffer.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_lookup_buffer
  copying build\lib\vllm\distributed\kv_transfer\kv_lookup_buffer\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_lookup_buffer
  creating build\bdist.win-amd64\wheel\vllm\distributed\kv_transfer\kv_pipe
  copying build\lib\vllm\distributed\kv_transfer\kv_pipe\base.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_pipe
  copying build\lib\vllm\distributed\kv_transfer\kv_pipe\mooncake_pipe.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_pipe
  copying build\lib\vllm\distributed\kv_transfer\kv_pipe\pynccl_pipe.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_pipe
  copying build\lib\vllm\distributed\kv_transfer\kv_pipe\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer\kv_pipe
  copying build\lib\vllm\distributed\kv_transfer\kv_transfer_state.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer
  copying build\lib\vllm\distributed\kv_transfer\README.md -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer
  copying build\lib\vllm\distributed\kv_transfer\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\distributed\kv_transfer
  copying build\lib\vllm\distributed\parallel_state.py -> build\bdist.win-amd64\wheel\.\vllm\distributed
  copying build\lib\vllm\distributed\tpu_distributed_utils.py -> build\bdist.win-amd64\wheel\.\vllm\distributed
  copying build\lib\vllm\distributed\utils.py -> build\bdist.win-amd64\wheel\.\vllm\distributed
  copying build\lib\vllm\distributed\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\distributed
  creating build\bdist.win-amd64\wheel\vllm\engine
  copying build\lib\vllm\engine\arg_utils.py -> build\bdist.win-amd64\wheel\.\vllm\engine
  copying build\lib\vllm\engine\async_llm_engine.py -> build\bdist.win-amd64\wheel\.\vllm\engine
  copying build\lib\vllm\engine\async_timeout.py -> build\bdist.win-amd64\wheel\.\vllm\engine
  copying build\lib\vllm\engine\llm_engine.py -> build\bdist.win-amd64\wheel\.\vllm\engine
  copying build\lib\vllm\engine\metrics.py -> build\bdist.win-amd64\wheel\.\vllm\engine
  copying build\lib\vllm\engine\metrics_types.py -> build\bdist.win-amd64\wheel\.\vllm\engine
  creating build\bdist.win-amd64\wheel\vllm\engine\multiprocessing
  copying build\lib\vllm\engine\multiprocessing\client.py -> build\bdist.win-amd64\wheel\.\vllm\engine\multiprocessing
  copying build\lib\vllm\engine\multiprocessing\engine.py -> build\bdist.win-amd64\wheel\.\vllm\engine\multiprocessing
  copying build\lib\vllm\engine\multiprocessing\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\engine\multiprocessing
  creating build\bdist.win-amd64\wheel\vllm\engine\output_processor
  copying build\lib\vllm\engine\output_processor\interfaces.py -> build\bdist.win-amd64\wheel\.\vllm\engine\output_processor
  copying build\lib\vllm\engine\output_processor\multi_step.py -> build\bdist.win-amd64\wheel\.\vllm\engine\output_processor
  copying build\lib\vllm\engine\output_processor\single_step.py -> build\bdist.win-amd64\wheel\.\vllm\engine\output_processor
  copying build\lib\vllm\engine\output_processor\stop_checker.py -> build\bdist.win-amd64\wheel\.\vllm\engine\output_processor
  copying build\lib\vllm\engine\output_processor\util.py -> build\bdist.win-amd64\wheel\.\vllm\engine\output_processor
  copying build\lib\vllm\engine\output_processor\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\engine\output_processor
  copying build\lib\vllm\engine\protocol.py -> build\bdist.win-amd64\wheel\.\vllm\engine
  copying build\lib\vllm\engine\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\engine
  creating build\bdist.win-amd64\wheel\vllm\entrypoints
  copying build\lib\vllm\entrypoints\api_server.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  copying build\lib\vllm\entrypoints\chat_utils.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  creating build\bdist.win-amd64\wheel\vllm\entrypoints\cli
  creating build\bdist.win-amd64\wheel\vllm\entrypoints\cli\benchmark
  copying build\lib\vllm\entrypoints\cli\benchmark\base.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli\benchmark
  copying build\lib\vllm\entrypoints\cli\benchmark\latency.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli\benchmark
  copying build\lib\vllm\entrypoints\cli\benchmark\main.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli\benchmark
  copying build\lib\vllm\entrypoints\cli\benchmark\serve.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli\benchmark
  copying build\lib\vllm\entrypoints\cli\benchmark\throughput.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli\benchmark
  copying build\lib\vllm\entrypoints\cli\benchmark\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli\benchmark
  copying build\lib\vllm\entrypoints\cli\collect_env.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli
  copying build\lib\vllm\entrypoints\cli\main.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli
  copying build\lib\vllm\entrypoints\cli\openai.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli
  copying build\lib\vllm\entrypoints\cli\run_batch.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli
  copying build\lib\vllm\entrypoints\cli\serve.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli
  copying build\lib\vllm\entrypoints\cli\types.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli
  copying build\lib\vllm\entrypoints\cli\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\cli
  copying build\lib\vllm\entrypoints\launcher.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  copying build\lib\vllm\entrypoints\llm.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  copying build\lib\vllm\entrypoints\logger.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  creating build\bdist.win-amd64\wheel\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\api_server.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\cli_args.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\logits_processors.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\protocol.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\run_batch.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_chat.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_classification.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_completion.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_embedding.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_engine.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_models.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_pooling.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_score.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_tokenization.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\openai\serving_transcription.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  creating build\bdist.win-amd64\wheel\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\abstract_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\deepseekv3_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\granite_20b_fc_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\granite_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\hermes_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\internlm2_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\jamba_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\llama4_pythonic_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\llama_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\mistral_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\phi4mini_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\pythonic_tool_parser.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\utils.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\tool_parsers\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai\tool_parsers
  copying build\lib\vllm\entrypoints\openai\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints\openai
  copying build\lib\vllm\entrypoints\score_utils.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  copying build\lib\vllm\entrypoints\ssl.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  copying build\lib\vllm\entrypoints\utils.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  copying build\lib\vllm\entrypoints\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\entrypoints
  copying build\lib\vllm\envs.py -> build\bdist.win-amd64\wheel\.\vllm
  copying build\lib\vllm\env_override.py -> build\bdist.win-amd64\wheel\.\vllm
  creating build\bdist.win-amd64\wheel\vllm\executor
  copying build\lib\vllm\executor\executor_base.py -> build\bdist.win-amd64\wheel\.\vllm\executor
  copying build\lib\vllm\executor\mp_distributed_executor.py -> build\bdist.win-amd64\wheel\.\vllm\executor
  copying build\lib\vllm\executor\msgspec_utils.py -> build\bdist.win-amd64\wheel\.\vllm\executor
  copying build\lib\vllm\executor\multiproc_worker_utils.py -> build\bdist.win-amd64\wheel\.\vllm\executor
  copying build\lib\vllm\executor\ray_distributed_executor.py -> build\bdist.win-amd64\wheel\.\vllm\executor
  copying build\lib\vllm\executor\ray_utils.py -> build\bdist.win-amd64\wheel\.\vllm\executor
  copying build\lib\vllm\executor\uniproc_executor.py -> build\bdist.win-amd64\wheel\.\vllm\executor
  copying build\lib\vllm\executor\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\executor
  copying build\lib\vllm\forward_context.py -> build\bdist.win-amd64\wheel\.\vllm
  creating build\bdist.win-amd64\wheel\vllm\inputs
  copying build\lib\vllm\inputs\data.py -> build\bdist.win-amd64\wheel\.\vllm\inputs
  copying build\lib\vllm\inputs\parse.py -> build\bdist.win-amd64\wheel\.\vllm\inputs
  copying build\lib\vllm\inputs\preprocess.py -> build\bdist.win-amd64\wheel\.\vllm\inputs
  copying build\lib\vllm\inputs\registry.py -> build\bdist.win-amd64\wheel\.\vllm\inputs
  copying build\lib\vllm\inputs\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\inputs
  copying build\lib\vllm\jsontree.py -> build\bdist.win-amd64\wheel\.\vllm
  copying build\lib\vllm\logger.py -> build\bdist.win-amd64\wheel\.\vllm
  creating build\bdist.win-amd64\wheel\vllm\logging_utils
  copying build\lib\vllm\logging_utils\dump_input.py -> build\bdist.win-amd64\wheel\.\vllm\logging_utils
  copying build\lib\vllm\logging_utils\formatter.py -> build\bdist.win-amd64\wheel\.\vllm\logging_utils
  copying build\lib\vllm\logging_utils\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\logging_utils
  copying build\lib\vllm\logits_process.py -> build\bdist.win-amd64\wheel\.\vllm
  creating build\bdist.win-amd64\wheel\vllm\lora
  copying build\lib\vllm\lora\fully_sharded_layers.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  copying build\lib\vllm\lora\layers.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  copying build\lib\vllm\lora\lora.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  copying build\lib\vllm\lora\models.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  creating build\bdist.win-amd64\wheel\vllm\lora\ops
  creating build\bdist.win-amd64\wheel\vllm\lora\ops\torch_ops
  copying build\lib\vllm\lora\ops\torch_ops\lora_ops.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\torch_ops
  copying build\lib\vllm\lora\ops\torch_ops\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\torch_ops
  creating build\bdist.win-amd64\wheel\vllm\lora\ops\triton_ops
  copying build\lib\vllm\lora\ops\triton_ops\kernel_utils.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\triton_ops
  copying build\lib\vllm\lora\ops\triton_ops\lora_expand_op.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\triton_ops
  copying build\lib\vllm\lora\ops\triton_ops\lora_kernel_metadata.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\triton_ops
  copying build\lib\vllm\lora\ops\triton_ops\lora_shrink_op.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\triton_ops
  copying build\lib\vllm\lora\ops\triton_ops\utils.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\triton_ops
  copying build\lib\vllm\lora\ops\triton_ops\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\triton_ops
  creating build\bdist.win-amd64\wheel\vllm\lora\ops\xla_ops
  copying build\lib\vllm\lora\ops\xla_ops\lora_ops.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\xla_ops
  copying build\lib\vllm\lora\ops\xla_ops\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops\xla_ops
  copying build\lib\vllm\lora\ops\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\lora\ops
  copying build\lib\vllm\lora\peft_helper.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  creating build\bdist.win-amd64\wheel\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\punica_wrapper\punica_base.py -> build\bdist.win-amd64\wheel\.\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\punica_wrapper\punica_cpu.py -> build\bdist.win-amd64\wheel\.\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\punica_wrapper\punica_gpu.py -> build\bdist.win-amd64\wheel\.\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\punica_wrapper\punica_hpu.py -> build\bdist.win-amd64\wheel\.\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\punica_wrapper\punica_selector.py -> build\bdist.win-amd64\wheel\.\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\punica_wrapper\punica_tpu.py -> build\bdist.win-amd64\wheel\.\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\punica_wrapper\utils.py -> build\bdist.win-amd64\wheel\.\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\punica_wrapper\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\lora\punica_wrapper
  copying build\lib\vllm\lora\request.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  copying build\lib\vllm\lora\resolver.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  copying build\lib\vllm\lora\utils.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  copying build\lib\vllm\lora\worker_manager.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  copying build\lib\vllm\lora\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\lora
  creating build\bdist.win-amd64\wheel\vllm\model_executor
  copying build\lib\vllm\model_executor\custom_op.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor
  creating build\bdist.win-amd64\wheel\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\guidance_decoding.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\guidance_logits_processors.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\guided_fields.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\lm_format_enforcer_decoding.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\outlines_decoding.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\outlines_logits_processors.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\utils.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\xgrammar_decoding.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  copying build\lib\vllm\model_executor\guided_decoding\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\guided_decoding
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers
  copying build\lib\vllm\model_executor\layers\activation.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\batched_deep_gemm_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\batched_triton_or_deep_gemm_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=14336,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=14336,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=1792,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=1792,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=3072,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=3072,device_name=NVIDIA_H100_80GB_HBM3,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=3072,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=3584,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=3584,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=1,N=7168,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=1024,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=1024,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_H20-3e.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_H20.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=192,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H20-3e.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H20.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=384,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=512,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=NVIDIA_H20.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=768,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=128,N=96,device_name=NVIDIA_H20.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=1024,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=1024,device_name=NVIDIA_H100.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=1344,device_name=NVIDIA_A100-SXM4-40GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=1344,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=1344,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=14336,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=14336,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=1792,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=1792,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=2688,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=2688,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=3072,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=3072,device_name=NVIDIA_H100_80GB_HBM3,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=3200,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=3584,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=3584,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=6400,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=7168,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=7168,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=int8_w8a16.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=16,N=800,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=160,N=192,device_name=NVIDIA_A800-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=1024,device_name=AMD_Instinct_MI325X,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=1024,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_A100-SXM4-80GB,dtype=int8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_A800-SXM4-80GB,dtype=int8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=128,device_name=NVIDIA_L20Y,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_B200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_H20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_H20,dtype=int8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_H20-3e,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_H200,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=256,device_name=NVIDIA_L20,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=512,device_name=AMD_Instinct_MI325_OAM,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=256,N=64,device_name=NVIDIA_A800-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=60,N=1408,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=60,N=176,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=60,N=352,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=60,N=704,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_A800-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=1280,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=2560,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=2560,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=2560,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=320,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=320,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=320,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=320,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_A800-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_GeForce_RTX_4090,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=640,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=64,N=896,device_name=NVIDIA_H20.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=AMD_Instinct_MI325X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=14336,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=16384,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=16384,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=16384,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=16384,device_name=AMD_Instinct_MI325X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=AMD_Instinct_MI325X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_A100-SXM4-40GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=1792,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=AMD_Instinct_MI325X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=2048,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=AMD_Instinct_MI325X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_A100-SXM4-40GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_GeForce_RTX_4090,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=3584,device_name=NVIDIA_L40S.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=AMD_Instinct_MI325X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=4096,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=AMD_Instinct_MI325X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_A100-SXM4-80GB.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_H100_80GB_HBM3.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=7168,device_name=NVIDIA_H200.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=AMD_Instinct_MI300X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=AMD_Instinct_MI325X,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=AMD_Instinct_MI325X.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=NVIDIA_H100_80GB_HBM3,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\E=8,N=8192,device_name=NVIDIA_H200,dtype=fp8_w8a8.json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\configs\README -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe\configs
  copying build\lib\vllm\model_executor\layers\fused_moe\cutlass_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\deepep_ht_prepare_finalize.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\deepep_ll_prepare_finalize.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\deep_gemm_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\fused_batched_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\fused_marlin_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\fused_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\layer.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\modular_kernel.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\moe_align_block_size.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\moe_pallas.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\moe_permute_unpermute.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\moe_torch_iterative.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\pplx_prepare_finalize.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\prepare_finalize.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\rocm_aiter_fused_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\triton_deep_gemm_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\utils.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\fused_moe\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\fused_moe
  copying build\lib\vllm\model_executor\layers\layernorm.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers
  copying build\lib\vllm\model_executor\layers\lightning_attn.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers
  copying build\lib\vllm\model_executor\layers\linear.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers
  copying build\lib\vllm\model_executor\layers\logits_processor.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\mamba
  copying build\lib\vllm\model_executor\layers\mamba\mamba2_metadata.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba
  copying build\lib\vllm\model_executor\layers\mamba\mamba_mixer.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba
  copying build\lib\vllm\model_executor\layers\mamba\mamba_mixer2.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\ops\causal_conv1d.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\ops\mamba_ssm.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\ops\ssd_bmm.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\ops\ssd_chunk_scan.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\ops\ssd_chunk_state.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\ops\ssd_combined.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\ops\ssd_state_passing.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\ops\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba\ops
  copying build\lib\vllm\model_executor\layers\mamba\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\mamba
  copying build\lib\vllm\model_executor\layers\pooler.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\aqlm.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\auto_round.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\awq.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\awq_marlin.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\awq_triton.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\base_config.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\bitblas.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\bitsandbytes.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\compressed_tensors
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\compressed_tensors.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\compressed_tensors_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_24.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_scheme.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w4a16_24.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w4a16_nvfp4.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w4a4_nvfp4.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w8a16_fp8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w8a8_fp8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_w8a8_int8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\compressed_tensors_wNa16.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\schemes\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors\schemes
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\triton_scaled_mm.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\utils.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors
  copying build\lib\vllm\model_executor\layers\quantization\compressed_tensors\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\compressed_tensors
  copying build\lib\vllm\model_executor\layers\quantization\deepspeedfp.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\experts_int8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\fbgemm_fp8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\fp8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\gguf.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\gptq.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\gptq_bitblas.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\gptq_marlin.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\gptq_marlin_24.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\hqq_marlin.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\ipex_quant.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\kernels
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision\allspark.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision\bitblas.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision\exllama.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision\machete.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision\marlin.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision\MPLinearKernel.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\mixed_precision
  copying build\lib\vllm\model_executor\layers\quantization\kernels\mixed_precision\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\mixed_precision
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm\aiter.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm\cutlass.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm\ScaledMMLinearKernel.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm\triton.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm\xla.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying build\lib\vllm\model_executor\layers\quantization\kernels\scaled_mm\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels\scaled_mm
  copying build\lib\vllm\model_executor\layers\quantization\kernels\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\kernels
  copying build\lib\vllm\model_executor\layers\quantization\kv_cache.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\marlin.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\modelopt.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\moe_wna16.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\neuron_quant.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\ptpc_fp8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\qqq.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\quark
  copying build\lib\vllm\model_executor\layers\quantization\quark\quark.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark
  copying build\lib\vllm\model_executor\layers\quantization\quark\quark_moe.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\quark\schemes
  copying build\lib\vllm\model_executor\layers\quantization\quark\schemes\quark_scheme.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark\schemes
  copying build\lib\vllm\model_executor\layers\quantization\quark\schemes\quark_w4a4_mxfp4.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark\schemes
  copying build\lib\vllm\model_executor\layers\quantization\quark\schemes\quark_w8a8_fp8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark\schemes
  copying build\lib\vllm\model_executor\layers\quantization\quark\schemes\quark_w8a8_int8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark\schemes
  copying build\lib\vllm\model_executor\layers\quantization\quark\schemes\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark\schemes
  copying build\lib\vllm\model_executor\layers\quantization\quark\utils.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark
  copying build\lib\vllm\model_executor\layers\quantization\quark\__init__.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\quark
  copying build\lib\vllm\model_executor\layers\quantization\schema.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\torchao.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  copying build\lib\vllm\model_executor\layers\quantization\tpu_int8.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\utils
  copying build\lib\vllm\model_executor\layers\quantization\utils\allspark_utils.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\utils
  copying build\lib\vllm\model_executor\layers\quantization\utils\bitblas_utils.py -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\utils
  creating build\bdist.win-amd64\wheel\vllm\model_executor\layers\quantization\utils\configs
  copying build\lib\vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json -> build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\utils\configs
  error: could not create 'build\bdist.win-amd64\wheel\.\vllm\model_executor\layers\quantization\utils\configs\N=1536,K=1536,device_name=AMD_Instinct_MI300X,dtype=fp8_w8a8,block_shape=[128,128].json': No such file or directory
  [end of output]
  
  note: This error originates from a subprocess, and is likely not a problem with pip.
  ERROR: Failed building wheel for vllm
ERROR: Failed to build installable wheels for some pyproject.toml based projects (vllm)

```
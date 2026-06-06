import torch
from TTS.api import TTS
from common import *

__XTTS_MODEL_NAME = "tts_models/multilingual/multi-dataset/xtts_v2"
# 获取设备 (GPU 或 CPU)
device = "cuda" if torch.cuda.is_available() else "cpu"

# 初始化 XTTS v2 模型
# 该模型会自动下载并加载到本地缓存
tts = TTS(__XTTS_MODEL_NAME).to(device)

def make_voice(article:Article):
    if article.voice_type != "xtts":
        raise ValueError("Invalid voice type for XTTS model")
    
    speaker_wav_path= f"speek/data{article.voice_seed}.wav"
    output_path = f"output/output_xtts_{article.user_id}_{article.project_id}_{article.article_id}_{article.voice_seed}.wav"
    tts.tts_to_file(
        text=article.content,
        file_path=output_path,
        speed=article.voice_speed,
        language="zh-cn",
        speaker_wav=speaker_wav_path
    )        

    return (True, output_path)
import torch
from TTS.api import TTS
from common import *

__BAKER_MODEL_NAME = "tts_models/zh-CN/baker/tacotron2-DDC-GST"
# 初始化专门的中文模型
device = "cuda" if torch.cuda.is_available() else "cpu"
tts = TTS(model_name=__BAKER_MODEL_NAME)

def make_voice(article:Article):
    if article.voice_type != "baker":
        raise ValueError("Invalid voice type for Baker model")
    
    output_path = f"output/output_baker_{article.user_id}_{article.project_id}_{article.article_id}.wav"

    # 生成语音
    tts.tts_to_file(
        text=article.content,
        file_path=output_path,
        speed=article.voice_speed,
    )        

    return (True, output_path)
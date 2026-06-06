from fastapi import FastAPI
import uvicorn
from common import *
import base64

app = FastAPI()

@app.get("/")
def read_root():
    return {"it's ok"}

@app.post("/voice/make")
def make_voice(article: Article) -> ResponseModel : 
    try:
        check_request(article)
        if article.voice_type == "baker":
            import coq.baker as baker
            (ret, output_path) = baker.make_voice(article)
            if not ret:
                return ResponseModel(message="语音生成失败", success=False, data={"error": "生成失败"})
            
            with open(output_path, "rb") as f:
                data = f.read()
                encoded_string = base64.b64encode(data).decode('utf-8')
            return  ResponseModel(message="语音生成成功", success=True, data={"buffer": encoded_string}) 
            
        elif article.voice_type == "xtts":
            import coq.xtts as xtts
            (ret, output_path) = xtts.make_voice(article)
            if not ret:
                return ResponseModel(message="语音生成失败", success=False, data={"error": "生成失败"})
            
            with open(output_path, "rb") as f:
                data = f.read()
                encoded_string = base64.b64encode(data).decode('utf-8')
            return  ResponseModel(message="语音生成成功", success=True, data={"buffer": encoded_string}) 
        
        raise ValueError("不支持的语音类型")
    except Exception as e:
        return ResponseModel(message="语音生成失败", success=False, data={"error": str(e)})
    

def check_request(article: Article):
    print(f"Received request: {article}")
    if not article.user_id:
        raise ValueError("用户ID不能为空")
    if not article.content:
        raise ValueError("文章内容不能为空")
    if article.voice_speed <= 0:
        raise ValueError("语速必须大于0")
    

if __name__ == "__main__":
    uvicorn.run("app:app", host="192.168.0.108", port=8090, reload=True)
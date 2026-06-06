from pydantic import BaseModel

class ResponseModel(BaseModel):
    message: str
    success: bool
    data: dict = None


class Article(BaseModel):
    user_id: int
    project_id: int
    article_id: int
    voice_type: str
    voice_seed: int = 1
    voice_speed: float = 1.5
    content: str


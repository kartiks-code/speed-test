FROM python:3.12

WORKDIR /app

COPY . .

# setup.cfg only declares fastapi[all]; the app also needs asyncpg etc.,
# so install the full pinned requirements and run from src via PYTHONPATH.
RUN pip install --no-cache-dir -r requirements.txt

ENV PYTHONPATH=/app/src

EXPOSE 8080

CMD ["uvicorn", "petstore.main:app", "--host", "0.0.0.0", "--port", "8080"]

#!/usr/bin/env python3
"""
Demo FastAPI backend for testing Chaos Testing framework
Run: uvicorn demo-api:app --port 9000
"""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Optional

app = FastAPI(title="Demo API")

class User(BaseModel):
    id: int
    name: str
    email: str

class Product(BaseModel):
    id: int
    name: str
    price: float

class Order(BaseModel):
    item: str
    quantity: int = 1

users_db = [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob", "email": "bob@example.com"},
]

products_db = [
    {"id": 1, "name": "Widget", "price": 9.99},
    {"id": 2, "name": "Gadget", "price": 19.99},
    {"id": 123, "name": "Special Item", "price": 99.99},
]

@app.get("/")
def read_root():
    return {"message": "Demo API is running", "version": "1.0.0"}

@app.get("/api/users")
def get_users():
    return {"users": users_db, "count": len(users_db)}

@app.get("/api/users/{user_id}")
def get_user(user_id: int):
    user = next((u for u in users_db if u["id"] == user_id), None)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    return user

@app.get("/api/products")
def get_products():
    return {"products": products_db, "count": len(products_db)}

@app.get("/api/products/{product_id}")
def get_product(product_id: int):
    product = next((p for p in products_db if p["id"] == product_id), None)
    if not product:
        raise HTTPException(status_code=404, detail="Product not found")
    return product

@app.post("/api/orders")
def create_order(order: Order):
    return {
        "success": True,
        "order_id": 42,
        "item": order.item,
        "quantity": order.quantity,
        "status": "processing"
    }

@app.get("/api/health")
def health_check():
    return {"status": "healthy"}

if __name__ == "__main__":
    import uvicorn
    print("Starting Demo API on http://localhost:9000")
    uvicorn.run(app, host="0.0.0.0", port=9000)

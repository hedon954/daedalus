from transformers import pipeline

classifier = pipeline("text-classification")

result = classifier(
    ["I've been waiting for a HuggingFace course my whole life.", "I hate this so much!"]
)
print(result)

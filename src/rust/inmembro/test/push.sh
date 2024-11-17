#!/usr/bin/env bash
curl -i -w "\n" -X POST 'http://localhost:42084/topic/some/push' -H 'Content-type: application/json' -d '{}'

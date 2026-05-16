#!/usr/bin/env python3
"""生成关卡 JSON 文件 - CellType 使用正确的外部标签枚举格式
   Empty/Wall/Bomb/HighValue -> JSON string
   Normal(Blue)/Normal(Red)  -> JSON object {"Normal":"Blue"} / {"Normal":"Red"}
"""

import json
import os

W = 60
H = 40

# CellType 的 Python 表示
EMPTY = "Empty"
WALL = "Wall"
BOMB = "Bomb"
HV = "HighValue"

def NB():
    """Normal(Blue) -> {"Normal":"Blue"}"""
    return {"Normal": "Blue"}

def NR():
    """Normal(Red) -> {"Normal":"Red"}"""
    return {"Normal": "Red"}


def make_grid():
    return [[EMPTY for _ in range(W)] for _ in range(H)]


def place_pattern(grid, pattern, dx, dy):
    for y, row in enumerate(pattern):
        for x, cell in enumerate(row):
            nx, ny = dx + x, dy + y
            if 0 <= nx < W and 0 <= ny < H:
                grid[ny][nx] = cell


def place_wall_line(grid, x1, y1, x2, y2):
    if x1 == x2:
        for y in range(min(y1, y2), max(y1, y2) + 1):
            if 0 <= x1 < W and 0 <= y < H:
                grid[y][x1] = WALL
    elif y1 == y2:
        for x in range(min(x1, x2), max(x1, x2) + 1):
            if 0 <= x < W and 0 <= y1 < H:
                grid[y1][x] = WALL


# ============================================================
# 关卡定义
# ============================================================

def gen_level_01():
    """初次接触 - 蓝方细胞群 + 墙壁屏障"""
    g = make_grid()
    n = NB()

    # Block (静态方块)
    place_pattern(g, [[n,n],[n,n]], 28, 8)

    # Glider (滑翔机 - 会移动)
    place_pattern(g, [
        [n, EMPTY, n],
        [EMPTY, n, n],
        [EMPTY, n, EMPTY],
    ], 42, 6)

    # 第二组 Block
    place_pattern(g, [[n,n],[n,n]], 15, 12)

    # Beehive (蜂巢)
    place_pattern(g, [
        [EMPTY, n, n, EMPTY],
        [n, EMPTY, EMPTY, n],
        [EMPTY, n, n, EMPTY],
    ], 35, 14)

    # 墙壁
    place_wall_line(g, 10, 20, 25, 20)
    place_wall_line(g, 30, 20, 45, 20)
    place_wall_line(g, 10, 15, 10, 22)

    # 炸弹
    g[9][29] = BOMB

    # 高价值目标
    g[19][20] = HV

    return g


def gen_level_02():
    """防守反击 - 更多蓝方细胞和复杂墙壁"""
    g = make_grid()
    n = NB()

    # 大型空心方块
    place_pattern(g, [
        [n,n,n],
        [n, EMPTY, n],
        [n,n,n],
    ], 25, 5)

    # 两个滑翔机
    place_pattern(g, [
        [n, EMPTY, n],
        [EMPTY, n, n],
        [EMPTY, n, EMPTY],
    ], 10, 4)
    place_pattern(g, [
        [n, EMPTY, n],
        [EMPTY, n, n],
        [EMPTY, n, EMPTY],
    ], 48, 4)

    # 两个蜂巢
    place_pattern(g, [
        [EMPTY, n, n, EMPTY],
        [n, EMPTY, EMPTY, n],
        [EMPTY, n, n, EMPTY],
    ], 18, 10)
    place_pattern(g, [
        [EMPTY, n, n, EMPTY],
        [n, EMPTY, EMPTY, n],
        [EMPTY, n, n, EMPTY],
    ], 40, 12)

    # 四组方块
    for bx, by in [(5, 8), (50, 8), (28, 14), (32, 14)]:
        place_pattern(g, [[n,n],[n,n]], bx, by)

    # 复杂墙壁
    place_wall_line(g, 5, 18, 20, 18)
    place_wall_line(g, 35, 18, 55, 18)
    place_wall_line(g, 5, 18, 5, 28)
    place_wall_line(g, 55, 18, 55, 28)
    place_wall_line(g, 5, 28, 55, 28)

    # 炸弹
    g[14][12] = BOMB
    g[14][48] = BOMB

    # 高价值目标
    g[27][19] = HV

    return g


def gen_level_03():
    """突围战 - 大量蓝方细胞组成的防线"""
    g = make_grid()
    n = NB()

    # 密集防线
    for x in range(10, 50, 3):
        for y in range(3, 8):
            if (x + y) % 4 in (0, 1):
                g[y][x] = n

    # 四个滑翔机
    for sx, sy in [(8, 3), (22, 3), (36, 3), (50, 3)]:
        place_pattern(g, [
            [n, EMPTY, n],
            [EMPTY, n, n],
            [EMPTY, n, EMPTY],
        ], sx, sy)

    # 三个空心方块
    for bx in [10, 28, 48]:
        place_pattern(g, [
            [n,n,n],
            [n, EMPTY, n],
            [n,n,n],
        ], bx, 12)

    # 三个蜂巢
    for bx, by in [(16, 10), (34, 10), (44, 10)]:
        place_pattern(g, [
            [EMPTY, n, n, EMPTY],
            [n, EMPTY, EMPTY, n],
            [EMPTY, n, n, EMPTY],
        ], bx, by)

    # 墙壁迷宫
    place_wall_line(g, 0, 18, 15, 18)
    place_wall_line(g, 20, 18, 35, 18)
    place_wall_line(g, 40, 18, 59, 18)
    place_wall_line(g, 15, 18, 15, 22)
    place_wall_line(g, 20, 18, 20, 22)
    place_wall_line(g, 35, 18, 35, 22)
    place_wall_line(g, 40, 18, 40, 22)
    place_wall_line(g, 0, 22, 59, 22)

    # 炸弹
    g[12][11] = BOMB
    g[12][30] = BOMB
    g[12][49] = BOMB

    # 高价值目标
    g[22][23] = HV
    g[22][37] = HV

    return g


def make_level(lid, name, grid, max_gliders=3, max_lwss=2, evo_steps=200):
    dep_zone = [{"x": 5 + dx, "y": 30 + dy} for dx in range(5) for dy in range(5)]
    return {
        "id": lid,
        "name": name,
        "version": 1,
        "width": W,
        "height": H,
        "max_gliders": max_gliders,
        "max_lwss": max_lwss,
        "evolution_steps": evo_steps,
        "deployment_zone": dep_zone,
        "initial_cells": grid,
    }


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_dir = os.path.dirname(script_dir)
    out_dir = os.path.join(project_dir, "assets", "levels")
    os.makedirs(out_dir, exist_ok=True)

    levels = [
        ("level_01", "初次接触", gen_level_01()),
        ("level_02", "防守反击", gen_level_02()),
        ("level_03", "突围战", gen_level_03()),
    ]

    for lid, name, grid in levels:
        data = make_level(lid, name, grid)
        path = os.path.join(out_dir, f"{lid}.json")
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
        print(f"Generated: {path}")
        # 验证: 检查输出中不包含字符串化的 Normal
        with open(path, "r") as f:
            content = f.read()
            if '"{"Normal":"Blue"}"' in content or r'"{\"Normal' in content:
                print(f"  ERROR: Normal cells are still strings!")
            else:
                print(f"  OK: Normal cells are proper JSON objects")

    print("Done!")


if __name__ == "__main__":
    main()

/**
 * CircularBuffer 单元测试
 */

import { describe, it, expect } from 'vitest';
import { CircularBuffer } from '../lib/CircularBuffer';

describe('CircularBuffer', () => {
  it('应该创建指定容量的缓冲区', () => {
    const buffer = new CircularBuffer<number>(5);
    expect(buffer.size).toBe(5);
    expect(buffer.length).toBe(0);
    expect(buffer.isEmpty).toBe(true);
    expect(buffer.isFull).toBe(false);
  });

  it('应该正确添加元素', () => {
    const buffer = new CircularBuffer<string>(3);
    buffer.push('a');
    buffer.push('b');

    expect(buffer.length).toBe(2);
    expect(buffer.toArray()).toEqual(['a', 'b']);
  });

  it('应该在满容量时覆盖最旧的元素', () => {
    const buffer = new CircularBuffer<number>(3);
    buffer.push(1);
    buffer.push(2);
    buffer.push(3);
    expect(buffer.isFull).toBe(true);

    // 添加第4个元素,应该覆盖第1个
    buffer.push(4);
    expect(buffer.length).toBe(3);
    expect(buffer.toArray()).toEqual([2, 3, 4]);

    // 继续添加
    buffer.push(5);
    buffer.push(6);
    expect(buffer.toArray()).toEqual([4, 5, 6]);
  });

  it('应该正确批量添加元素', () => {
    const buffer = new CircularBuffer<string>(5);
    buffer.pushMany(['a', 'b', 'c']);

    expect(buffer.length).toBe(3);
    expect(buffer.toArray()).toEqual(['a', 'b', 'c']);

    // 批量添加超过容量
    buffer.pushMany(['d', 'e', 'f', 'g']);
    expect(buffer.length).toBe(5);
    expect(buffer.toArray()).toEqual(['c', 'd', 'e', 'f', 'g']);
  });

  it('应该正确清空缓冲区', () => {
    const buffer = new CircularBuffer<number>(5);
    buffer.pushMany([1, 2, 3, 4, 5]);

    buffer.clear();
    expect(buffer.isEmpty).toBe(true);
    expect(buffer.length).toBe(0);
    expect(buffer.toArray()).toEqual([]);

    // 清空后应该可以重新添加
    buffer.push(10);
    expect(buffer.toArray()).toEqual([10]);
  });

  it('应该正确处理对象类型', () => {
    interface LogEntry {
      text: string;
      level: string;
    }

    const buffer = new CircularBuffer<LogEntry>(3);
    buffer.push({ text: 'log1', level: 'info' });
    buffer.push({ text: 'log2', level: 'error' });

    const result = buffer.toArray();
    expect(result).toHaveLength(2);
    expect(result[0]).toEqual({ text: 'log1', level: 'info' });
    expect(result[1]).toEqual({ text: 'log2', level: 'error' });
  });

  it('边界测试: 容量为1', () => {
    const buffer = new CircularBuffer<number>(1);
    buffer.push(1);
    expect(buffer.toArray()).toEqual([1]);

    buffer.push(2);
    expect(buffer.toArray()).toEqual([2]);

    buffer.push(3);
    expect(buffer.toArray()).toEqual([3]);
  });

  it('性能测试: 大量元素', () => {
    const buffer = new CircularBuffer<number>(500);
    const startTime = performance.now();

    // 添加1000个元素(会覆盖500个旧元素)
    for (let i = 0; i < 1000; i++) {
      buffer.push(i);
    }

    const endTime = performance.now();
    const duration = endTime - startTime;

    expect(buffer.length).toBe(500);
    expect(buffer.toArray()[0]).toBe(500);
    expect(buffer.toArray()[499]).toBe(999);

    // 性能断言: 应该在100ms内完成
    expect(duration).toBeLessThan(100);
  });
});

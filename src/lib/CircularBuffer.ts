/**
 * 循环缓冲区实现
 * 用于高效管理固定大小的日志队列,避免频繁的数组 splice 操作
 */

export class CircularBuffer<T> {
  private buffer: (T | undefined)[];
  private head = 0;
  private tail = 0;
  private count = 0;

  constructor(private capacity: number) {
    this.buffer = new Array(capacity);
  }

  /** 添加元素到缓冲区末尾 */
  push(item: T): void {
    this.buffer[this.tail] = item;
    this.tail = (this.tail + 1) % this.capacity;

    if (this.count < this.capacity) {
      this.count++;
    } else {
      // 缓冲区已满,head 向前移动覆盖最旧的元素
      this.head = (this.head + 1) % this.capacity;
    }
  }

  /** 批量添加元素 */
  pushMany(items: T[]): void {
    for (const item of items) {
      this.push(item);
    }
  }

  /** 获取所有有效元素(按插入顺序) */
  toArray(): T[] {
    if (this.count === 0) return [];

    const result: T[] = [];
    let idx = this.head;
    for (let i = 0; i < this.count; i++) {
      const item = this.buffer[idx];
      if (item !== undefined) {
        result.push(item);
      }
      idx = (idx + 1) % this.capacity;
    }
    return result;
  }

  /** 清空缓冲区 */
  clear(): void {
    this.head = 0;
    this.tail = 0;
    this.count = 0;
    this.buffer.fill(undefined);
  }

  /** 获取当前元素数量 */
  get length(): number {
    return this.count;
  }

  /** 获取缓冲区容量 */
  get size(): number {
    return this.capacity;
  }

  /** 检查缓冲区是否为空 */
  get isEmpty(): boolean {
    return this.count === 0;
  }

  /** 检查缓冲区是否已满 */
  get isFull(): boolean {
    return this.count === this.capacity;
  }
}

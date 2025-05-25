#[cfg(not(feature = "std"))]
extern crate alloc;

use core::task::Poll;

#[cfg(not(feature = "std"))]
use alloc::collections::{BTreeMap, VecDeque};

#[cfg(feature = "std")]
use std::collections::{HashMap, VecDeque};

/// 表示一个任务槽位
struct Slot<T> {
  /// 任务到期的轮数
  round: u128,
  /// 任务值
  value: T,
}

pub struct TimeWheel<T> {
  /// 使用 BTreeMap 存储每个时间槽位的任务队列
  #[cfg(not(feature = "std"))]
  hashed: BTreeMap<u128, VecDeque<Slot<T>>>,
  /// 使用哈希表存储每个时间槽位的任务队列
  #[cfg(feature = "std")]
  hashed: HashMap<u128, VecDeque<Slot<T>>>,
  /// 时间轮的总槽数
  pub steps: u128,
  /// 当前时间轮的刻度位置
  pub tick: u128,
}

impl<T> TimeWheel<T> {
  // create new hashed time wheel instance
  pub fn new(steps: u128) -> Self {
    assert!(steps > 0, "steps must be greater than 0");
    Self { steps, hashed: Default::default(), tick: 0 }
  }

  /// 向时间轮添加一个新任务，参数 timeout 表示任务的超时时间，value 是任务的具体内容。
  ///
  /// # Returns:
  /// - 返回该任务所在的槽位编号(Vec索引)。
  pub fn add(&mut self, timeout: u128, value: T) -> u128 {
    assert!(timeout > 0, "timeout must be greater than 0");

    // 获得任务（value）需要加入的槽位编号（Vec索引）
    let slot = if timeout.wrapping_add(self.tick) < timeout {
      panic!("Overflow detected in slot calculation");
    } else {
      (timeout + self.tick) % self.steps
    };

    // 获取或创建槽位对应的任务列表
    let slots = self.hashed.entry(slot).or_default();

    // 超时时间除以槽位数量，得到超时时间对应的轮数
    let round = timeout / self.steps;

    slots.push_back(Slot { value, round });

    if let Some(i) = slots.iter().position(|s| s.round > round) {
      slots.swap(i, slots.len() - 1);
    }

    slot
  }

  /// 从指定槽位移除一个任务，如果成功移除则返回 true
  pub fn remove(&mut self, slot: u128, value: &T) -> bool
  where
    T: PartialEq,
  {
    if let Some(slots) = self.hashed.get_mut(&slot) {
      if let Some(index) = slots.iter().position(|v| &v.value == value) {
        slots.remove(index);
        // slots.swap_remove_back(index);
        return true;
      }
    }

    false
  }

  /// 模拟时间轮向前推进一格，如果当前槽位有到期任务，则返回这些任务；否则返回 Poll::Pending 表示没有到期任务
  pub fn tick(&mut self) -> Poll<VecDeque<T>> {
    // 计算当前槽位编号
    let step = self.tick % self.steps;

    // 更新时间轮状态。使用 wrapping_add 避免数据溢出，`200u128.wrapping_add(u128::MAX) == 199`
    self.tick = self.tick.wrapping_add(1);

    // 检查当前槽位是否有任务
    if let Some(slots) = self.hashed.remove(&step) {
      // 分割出到期任务队列和未到期任务队列
      let mut current: VecDeque<T> = VecDeque::new();
      let mut reserved: VecDeque<Slot<T>> = VecDeque::new();

      for mut slot in slots {
        if slot.round == 0 {
          current.push_back(slot.value);
        } else {
          slot.round -= 1;
          reserved.push_back(slot);
        }
      }

      // 将未到期任务重新存入槽位
      if !reserved.is_empty() {
        self.hashed.insert(step, reserved);
      }

      return Poll::Ready(current);
    }

    Poll::Pending
  }
}

#[cfg(test)]
mod tests {
  use super::TimeWheel;
  use chrono::Local;
  use core::task::Poll;

  /// 测试移除任务功能
  #[test]
  fn test_remove() {
    let mut time_wheel = TimeWheel::new(1024);

    let slot = time_wheel.add(10, 1);

    assert!(time_wheel.remove(slot, &1));

    assert!(!time_wheel.remove(slot, &10));
  }

  /// 测试时间轮的 tick 方法，并验证任务到期情况
  #[test]
  fn test_issue_1() {
    let mut time_wheel = TimeWheel::new(2048);

    for _ in 0..34 {
      let _ = time_wheel.tick();
    }

    let slot = time_wheel.add(998, ());

    // 998 + 34 = 1032
    assert_eq!(slot, 1032);

    assert_eq!(time_wheel.tick, 34);

    let mut r = Poll::Pending;

    for _ in 0..999 {
      r = time_wheel.tick();
      match &r {
        Poll::Ready(tasks) => println!("[{}] tick:{} 到期任务：{:?}", Local::now(), time_wheel.tick, tasks),
        Poll::Pending => println!("[{}] tick:{} 暂无到期任务", Local::now(), time_wheel.tick),
      };
    }

    assert!(r.is_ready());
  }
}

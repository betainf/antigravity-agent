import React, { useState, useRef, useEffect } from 'react';

interface VibratingButtonProps {
  children: React.ReactNode;
  isActive: boolean;
  onComplete: () => void;
  totalDuration: number;
  vibrationStartDelay: number;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * 渐进式震动按钮组件
 * 0.5秒后开始震动，震动强度逐渐增大直到导出触发
 */
export const VibratingButton: React.FC<VibratingButtonProps> = ({
  children,
  isActive,
  onComplete,
  totalDuration,
  vibrationStartDelay,
  className = '',
  style = {}
}) => {
  const [vibrationIntensity, setVibrationIntensity] = useState(0);
  const [isVibrating, setIsVibrating] = useState(false);
  const startTimeRef = useRef<number>(0);
  const animationFrameRef = useRef<number | null>(null);
  const vibrationIntervalRef = useRef<number | null>(null);

  // 清理所有定时器和动画
  const cleanup = () => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
    if (vibrationIntervalRef.current) {
      clearInterval(vibrationIntervalRef.current);
      vibrationIntervalRef.current = null;
    }
  };

  // 重置状态
  const reset = () => {
    setVibrationIntensity(0);
    setIsVibrating(false);
    cleanup();
  };

  // 开始震动效果
  const startVibration = () => {
    setIsVibrating(true);

    // 创建震动动画循环
    const animate = () => {
      const elapsed = Date.now() - startTimeRef.current - vibrationStartDelay;
      const remainingTime = totalDuration - vibrationStartDelay;

      // 计算当前震动强度 (0% 到 100%)
      const progress = Math.min(elapsed / remainingTime, 1);
      const newIntensity = Math.floor(progress * 100);

      setVibrationIntensity(newIntensity);

      // 如果还在震动周期内，继续动画
      if (elapsed < remainingTime) {
        animationFrameRef.current = requestAnimationFrame(animate);
      } else {
        // 震动完成，触发导出
        console.log('✅ 震动动画完成，强度:', newIntensity, '%');
        setVibrationIntensity(100);
        setTimeout(() => {
          console.log('🎯 调用 onComplete 回调');
          onComplete();
          reset();
        }, 100);
      }
    };

    animationFrameRef.current = requestAnimationFrame(animate);
  };

  // 监听激活状态
  useEffect(() => {
    if (isActive) {
      console.log('🔄 震动组件激活，开始倒计时');
      startTimeRef.current = Date.now();

      // 设置震动开始延迟
      vibrationIntervalRef.current = setTimeout(() => {
        console.log('📳 开始震动效果');
        startVibration();
      }, vibrationStartDelay);
    } else {
      reset();
    }

    return cleanup;
  }, [isActive, totalDuration, vibrationStartDelay, onComplete]);

  // 计算震动偏移 - 使用缓动函数实现平滑加速度
  const getVibrationTransform = () => {
    if (!isVibrating || vibrationIntensity === 0) return '';

    const time = Date.now() / 150; // 稍微减慢动画速度
    const baseIntensity = vibrationIntensity / 100;

    // 使用缓动函数实现平滑的加速度曲线
    const easeOutQuart = (t: number) => 1 - Math.pow(1 - t, 4);
    const smoothIntensity = easeOutQuart(baseIntensity);

    // 最大偏移根据强度动态调整，但更加平滑
    const maxOffset = smoothIntensity * 1.5; // 最大偏移1.5px

    // 使用不同频率和相位创建自然震动
    const offsetX = Math.sin(time) * maxOffset;
    const offsetY = Math.sin(time * 1.4 + Math.PI / 4) * maxOffset * 0.8;

    // 轻微的旋转，更加微妙
    const rotation = Math.sin(time * 0.6 + Math.PI / 3) * maxOffset * 0.2;

    return `translate(${offsetX}px, ${offsetY}px) rotate(${rotation}deg)`;
  };

  // 计算透明度脉动效果 - 更平滑的脉动
  const getOpacity = () => {
    if (!isVibrating) return 1;

    const time = Date.now() / 300; // 更慢的脉动速度
    const baseIntensity = vibrationIntensity / 100;

    // 使用缓动函数让透明度变化更自然
    const easeInOutSine = (t: number) => -(Math.cos(Math.PI * t) - 1) / 2;
    const smoothIntensity = easeInOutSine(baseIntensity);

    // 更轻微的透明度变化，避免突兀
    const baseOpacity = 1 - smoothIntensity * 0.1; // 最高降低到90%透明度
    const pulse = Math.sin(time) * 0.03; // 更小的脉动幅度

    return Math.max(0.85, Math.min(1, baseOpacity + pulse));
  };

  return (
    <div
      className={`inline-block cursor-pointer ${className}`}
      style={{
        transform: getVibrationTransform(),
        opacity: getOpacity(),
        // 更自然的亮度变化，避免过度饱和
        filter: isVibrating ? `brightness(${1 + vibrationIntensity / 400})` : 'none',
        // 更快的过渡时间，让动画更流畅
        transition: 'transform 0.03s ease-out, opacity 0.05s ease-out, filter 0.08s ease-out',
        // 确保不会出现奇怪的边框或阴影
        boxShadow: 'none',
        outline: 'none',
        border: 'none',
        ...style
      }}
    >
      {children}
    </div>
  );
};
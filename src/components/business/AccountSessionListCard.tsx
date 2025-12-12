import React from 'react';
import { motion } from 'framer-motion';
import { cn } from "@/lib/utils.ts"; // 假设你有一个用于合并类名的工具函数
import { Button, Space } from "antd";
import { BaseAvatar } from "@/components/base-ui/BaseAvatar.tsx"; // 假设你的头像组件路径
import { Crown, Gem } from 'lucide-react';

type UserTier = 'free-tier' | 'g1-pro-tier' | 'g1-ultra-tier';

interface UserSessionCardProps {
  nickName: string;
  userAvatar: string;
  email: string;
  tier: UserTier; // 核心属性：用户级别
  // 0-1, -1 代表未知
  geminiQuota: number | -1;
  // 0-1, -1 代表未知
  claudeQuota: number | -1;
  isCurrentUser: boolean;
  onSelect: () => void
  onSwitch: () => void
  onDelete: () => void
}

// ==========================================
// 核心样式定义区域 (CSS-in-JS)
// ==========================================

// 定义需要动态注入的 CSS 属性类型
type TierVisualStyles = Pick<React.CSSProperties, 'background' | 'borderColor' | 'boxShadow' | 'backdropFilter' | 'WebkitBackdropFilter'>;

// 定义不同级别的视觉样式映射
const tierVisualStyles: Record<UserTier, TierVisualStyles> = {
  "free-tier": {
    // 【修改点】不再是纯白，而是极淡的灰白渐变，增加质感
    background: '', // slate-50 to white
    borderColor: '#e2e8f0', // slate-200
    // 保持干净、轻微的悬浮阴影
    boxShadow: '',
  },
  "g1-pro-tier": {
    // 顶部暖金渐变
    background: 'linear-gradient(to bottom, rgba(255, 251, 235, 0.95), rgba(255, 255, 255, 0.6))',
    borderColor: 'rgba(252, 211, 77, 0.7)', // amber-300ish
    // 暖金色光晕阴影
    boxShadow: '0 20px 40px -10px rgba(251, 191, 36, 0.25), 0 10px 20px -5px rgba(251, 191, 36, 0.1), inset 0 0 20px -10px rgba(251, 191, 36, 0.1)',
  },
  "g1-ultra-tier": {
    // 图片中效果的关键：左上角发散的极光径向渐变
    background: 'radial-gradient(ellipse at top left, rgba(233, 213, 255, 0.75), rgba(245, 208, 254, 0.5), rgba(207, 250, 254, 0.3))',
    borderColor: 'rgba(167, 139, 250, 0.8)', // violet-400ish
    // 图片中效果的关键：向四周强烈发散的紫青色辉光阴影
    boxShadow: '0 0 60px -15px rgba(139, 92, 246, 0.4), 0 20px 40px -10px rgba(139, 92, 246, 0.2), inset 0 0 30px -15px rgba(233, 213, 255, 0.5)',
    // 毛玻璃效果 (需要父容器有背景才能看出通透感)
    backdropFilter: 'blur(12px)',
    WebkitBackdropFilter: 'blur(12px)',
  },
};

// 定义不同级别的角标组件映射
const tierBadgeMap: Record<UserTier, React.ReactNode> = {
  "free-tier": <span className="px-1.5 py-0.5 bg-slate-100 text-slate-500 text-[10px] font-bold rounded-md leading-none border border-slate-200 shadow-sm">Free</span>,
  "g1-pro-tier": <span className="px-1.5 py-0.5 bg-amber-50 text-amber-600 text-[10px] font-bold rounded-md leading-none border border-amber-200/60 flex items-center gap-0.5 shadow-sm"><Crown size={10} className="fill-current" />Pro</span>,
  // Ultra 级别的角标样式
  "g1-ultra-tier": <span className="px-1.5 py-0.5 bg-violet-50 text-violet-600 text-[10px] font-bold rounded-md leading-none border border-violet-200/60 flex items-center gap-0.5 shadow-sm"><Gem size={10} className="fill-current" />Ultra</span>,
};


export function AccountSessionListCard(props: UserSessionCardProps) {
  const {tier} = props;

  // 获取当前级别的动态样式
  const currentVisualStyles = tierVisualStyles[tier];

  return (
    <motion.div
      onClick={props.onSelect}
      // 1. 基础 Tailwind 类：负责布局、内边距、圆角和过渡
      className={cn(
        "w-[320px] rounded-2xl p-6 border cursor-pointer transition-all duration-300 hover:-translate-y-1",
        // 添加一个通用的 hover 悬浮阴影叠加
        "hover:shadow-xl"
      )}
      // 2. 核心：使用 style 属性注入复杂的背景和阴影样式
      style={{
        ...currentVisualStyles,
      }}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      {/* 头部区域 */}
      <header className="flex items-center gap-4 mb-4 relative">
        <BaseAvatar
          className={cn(
            "h-12 w-12 rounded-full object-cover border-2 transition-all duration-300 shrink-0 ring-2 ring-offset-2",
            // 微调 Ultra 模式下头像的边框，让它更通透，配合背景
            tier === 'g1-ultra-tier'
              ? "border-white/60 ring-white/20"
              : props.isCurrentUser
                ? "border-blue-400 ring-blue-100"
                : "border-gray-200 ring-gray-50 group-hover:border-blue-300 group-hover:ring-blue-50"
          )}
          src={props.userAvatar}
          alt={props.nickName}
        />

        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h2 className="text-lg font-bold text-slate-900 leading-tight">{props.nickName}</h2>
            {/* 动态显示级别角标 */}
            <div className="mt-0.5">
              {tierBadgeMap[tier]}
            </div>
          </div>
          <p className="text-sm text-slate-500 font-medium">{props.email}</p>
        </div>

        {/* 当前用户指示器 */}
        {
          props.isCurrentUser && (
            <div className="absolute top-0 right-0 px-2 py-0.5 bg-blue-500 text-white text-[10px] font-bold rounded-full shadow-sm leading-tight z-10">
              当前
            </div>
          )
        }
      </header>

      {/* 进度条区域 */}
      {
        props.geminiQuota === -1
          ? <div className="space-y-3">
            <UsageItem
              label="Gemini"
              percentage={-1}
              color="bg-blue-400"
              // Ultra 模式下轨道颜色稍微透明一点，融合背景
              trackColor={tier === 'g1-ultra-tier' ? "bg-blue-100/40" : "bg-blue-50"}
            />
            <UsageItem
              label="Claude"
              percentage={-1}
              color="bg-violet-400"
              trackColor={tier === 'g1-ultra-tier' ? "bg-violet-100/40" : "bg-violet-50"}
            />
          </div>

          : <div className="space-y-3">
            <UsageItem
              label="Gemini"
              percentage={props.geminiQuota}
              color="bg-blue-400"
              trackColor={tier === 'g1-ultra-tier' ? "bg-blue-100/40" : "bg-blue-50"}
            />
            <UsageItem
              label="Claude"
              percentage={props.claudeQuota}
              color="bg-violet-400"
              trackColor={tier === 'g1-ultra-tier' ? "bg-violet-100/40" : "bg-violet-50"}
            />
          </div>
      }

      {/* 底部交互区域 */}
      <div className="mt-6 flex items-center justify-center relative">
        <Space.Compact>
          <Button
            onClick={e => {
              e.stopPropagation();
              props.onSwitch()
            }}
            disabled={props.isCurrentUser}
            type="primary"
          >
            切换
          </Button>
          <Button
            danger
            onClick={e => {
              e.stopPropagation()
              props.onDelete()
            }}
            disabled={props.isCurrentUser}
          >
            删除
          </Button>
        </Space.Compact>
      </div>
    </motion.div>
  );
}

// ==========================================
// 子组件：进度条 (保持不变)
// ==========================================
function UsageItem({ label, percentage, color, trackColor }: {
  label: string,
  percentage: number,
  color: string,
  trackColor: string
}) {
  const isUnknown = percentage === -1;
  const displayPercentage = isUnknown ? 0 : Math.round(percentage * 100);

  return (
    <div className="group">
      <div className="flex justify-between mb-2 text-sm">
        <span className="text-slate-700 font-medium">{label}</span>
        <span className="text-slate-400 font-mono tabular-nums">
          {isUnknown ? 'Unknown' : `${displayPercentage}%`}
        </span>
      </div>
      {/* 进度条轨道 */}
      <div className={cn("h-2.5 w-full rounded-full overflow-hidden transition-colors duration-300", trackColor)}>
        {/* 进度条主体 */}
        <motion.div
          className={cn("h-full rounded-full shadow-sm", color)}
          initial={{width: 0}}
          animate={{width: `${displayPercentage}%`}}
          transition={{type: "spring", stiffness: 40, damping: 12, delay: 0.2}}
        />
      </div>
    </div>
  );
}

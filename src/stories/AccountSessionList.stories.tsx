import type { Meta, StoryObj } from '@storybook/react';
import { fn } from 'storybook/test';
import { AccountSessionList, AccountSessionListAccountItem } from '@/components/business/AccountSessionList.tsx';
import { useAppSettings } from '@/modules/use-app-settings.ts';

const meta = {
  title: 'Business/AccountSessionList',
  component: AccountSessionList,
  parameters: {
    // 使用 padded 布局，这样列表不会紧贴边缘，更接近真实页面效果
    layout: 'padded',
    backgrounds: {
      default: 'light-gray',
      values: [
        { name: 'light-gray', value: '#f8fafc' },
        { name: 'dark', value: '#0f172a' },
      ],
    },
  },
  tags: ['autodocs'],
  argTypes: {
    accounts: {
      description: '用户账户列表',
      control: 'object',
    },
    currentUserEmail: {
      description: '当前登录用户的 Email',
      control: 'text',
    },
  },
  args: {
    // 模拟事件回调
    onSelect: fn(),
    onSwitch: fn(),
    onDelete: fn(),
  },
} satisfies Meta<typeof AccountSessionList>;

export default meta;
type Story = StoryObj<typeof meta>;

// --- Mock Data 生成器 ---
// 这里传入的是"未脱敏"的数据，组件内部会处理脱敏
const mockAccounts: AccountSessionListAccountItem[] = [
  {
    nickName: 'Admin User',
    email: 'admin.ops@company.com',
    userAvatar: 'https://api.dicebear.com/9.x/avataaars/svg?seed=Admin',
    geminiQuota: 0.85,
    claudeQuota: 0.92,
    tier: 'g1-pro-tier',
    apiKey: 'sk_mock_admin',
  },
  {
    nickName: 'Jason Bourne',
    email: 'jason.bourne@cia.gov',
    userAvatar: 'https://api.dicebear.com/9.x/avataaars/svg?seed=Jason',
    geminiQuota: 0.15, // 低配额
    claudeQuota: 0.4,
    tier: 'free-tier',
    apiKey: 'sk_mock_jason',
  },
  {
    nickName: 'Unknown Guest',
    email: 'guest.temp@provider.net',
    userAvatar: 'https://api.dicebear.com/9.x/avataaars/svg?seed=Guest',
    geminiQuota: -1, // 未知配额
    claudeQuota: 0.05, // 极低配额
    tier: 'g1-ultra-tier',
    apiKey: 'sk_mock_guest',
  },
];

const gridAccounts: AccountSessionListAccountItem[] = Array.from({ length: 9 }).map((_, i) => {
  const base = mockAccounts[i % mockAccounts.length];
  const [name, domain] = base.email.split('@');
  return {
    ...base,
    email: `${name}+${i}@${domain}`, // 生成唯一 email，避免 React key 冲突
    nickName: `${base.nickName} #${i + 1}`,
    apiKey: `${base.apiKey}_${i}`,
  };
});

const longEmailAccounts: AccountSessionListAccountItem[] = [
  {
    nickName: 'ThisIsAnExcessivelyLongNickName_ToTest_TextOverflow_AndLayoutStability_InUserCardHeader',
    email: 'this.is.a.super.long.email.address.with.many.sections.and.tags+storybook-overflow-test@subdomain1.subdomain2.subdomain3.subdomain4.some-very-long-company-domain.example.corp.company.com',
    userAvatar: 'https://api.dicebear.com/9.x/avataaars/svg?seed=LongEmail',
    geminiQuota: 0.66,
    claudeQuota: 0.77,
    tier: 'g1-pro-tier',
    apiKey: 'sk_mock_long_email',
  },
  ...mockAccounts,
];

// --- Stories ---

/**
 * 默认状态：展示列表数据。
 * 注意：邮箱和昵称应该会被组件内的 mask 工具自动脱敏。
 */
export const Default: Story = {
  render: (args) => {
    useAppSettings.setState({ privateMode: true });
    return <AccountSessionList {...args} />;
  },
  args: {
    accounts: mockAccounts,
  },
};

/**
 * 高亮当前用户：
 * currentUserEmail 命中某个账号时，该卡片应显示“当前”徽标并禁用操作。
 */
export const WithCurrentUser: Story = {
  render: (args) => {
    useAppSettings.setState({ privateMode: true });
    return <AccountSessionList {...args} />;
  },
  args: {
    accounts: mockAccounts,
    currentUserEmail: mockAccounts[0].email,
  },
};

/**
 * 空状态：
 * 当 accounts 为空数组时，应该显示 SVG 图标和提示文案。
 */
export const EmptyState: Story = {
  render: (args) => {
    useAppSettings.setState({ privateMode: true });
    return <AccountSessionList {...args} />;
  },
  args: {
    accounts: [],
  },
};

/**
 * 大数据量布局测试：
 * 用于测试 flex-wrap 是否正常工作，以及卡片间距是否合适。
 */
export const GridWrapLayout: Story = {
  render: (args) => {
    useAppSettings.setState({ privateMode: true });
    return <AccountSessionList {...args} />;
  },
  args: {
    accounts: gridAccounts,
    currentUserEmail: gridAccounts[0].email,
  },
};

/**
 * 过长邮箱/昵称：
 * 用于复现 header 文本溢出问题（privateMode=false 显示全量文本）。
 */
export const LongEmailOverflow: Story = {
  render: (args) => {
    useAppSettings.setState({ privateMode: false });
    return <AccountSessionList {...args} />;
  },
  args: {
    accounts: longEmailAccounts,
    currentUserEmail: longEmailAccounts[0].email,
  },
};

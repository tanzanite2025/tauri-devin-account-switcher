export interface Account {
  id: string;
  name: string;
  email?: string;
  password?: string;
  token?: string;
  org_id?: string;
  plan_tier: string;
  created_at: number;
}

export enum ContractStatus {
  opened = "Opened",
  sent = "Sent",
  complete = "Complete",
}

export enum StepperStatus {
  current = "current",
  upcoming = "upcoming",
  complete = "complete",
}

export interface IContract {
  title: string;
  description: string;
  status: string;
  expiration: string;
  cost: number;
  createdOn: string;
  id: string;
  author: string;
}

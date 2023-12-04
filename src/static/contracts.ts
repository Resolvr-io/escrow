import { IContract } from "~/lib/constants";
// Dummy Data for contracts
const contracts: IContract[] = [
  {
    author: "alice",
    title: "Contract 1",
    description: "This is a description",
    status: "Opened",
    createdOn: "2021-01-01",
    cost: 100000,
    expiration: "2021-01-02",
    id: "0",
  },
  {
    author: "bob",
    title: "Contract 2",
    description: "This is a description 2",
    status: "Sent",
    createdOn: "2022-01-01",
    cost: 100000,
    expiration: "2022-01-02",
    id: "1",
  },

  {
    author: "charlie",
    title: "Contract 3",
    description: "This is a description",
    status: "Complete",
    createdOn: "2023-01-01",
    cost: 100000,
    expiration: "2023-01-02",
    id: "2",
  },
  {
    author: "erica",
    title: "Contract 4",
    description: "This is a description",
    status: "Complete",
    createdOn: "2023-01-01",
    cost: 600000,
    expiration: "2023-01-02",
    id: "3",
  },
];
export default contracts;

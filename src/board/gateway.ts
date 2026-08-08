import { invoke } from "@tauri-apps/api/core";
import type {
  AddDependencyRequest,
  BoardGateway,
  BoardSnapshot,
  CreateBoardRequest,
  CreateProjectRequest,
  CreateWorkItemRequest,
  TransitionWorkItemRequest,
} from "./types";

export const tauriBoardGateway: BoardGateway = {
  async createProject(request: CreateProjectRequest): Promise<void> {
    await invoke("create_project", { request });
  },
  createBoard(request: CreateBoardRequest): Promise<BoardSnapshot> {
    return invoke("create_board", { request });
  },
  createWorkItem(request: CreateWorkItemRequest): Promise<BoardSnapshot> {
    return invoke("create_work_item", { request });
  },
  addDependency(request: AddDependencyRequest): Promise<BoardSnapshot> {
    return invoke("add_dependency", { request });
  },
  transitionWorkItem(
    request: TransitionWorkItemRequest,
  ): Promise<BoardSnapshot> {
    return invoke("transition_work_item", { request });
  },
  boardSnapshot(boardId: string): Promise<BoardSnapshot> {
    return invoke("board_snapshot", { boardId });
  },
};

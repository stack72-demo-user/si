import { registry } from "@/registry";
import { EntityObject } from "@/systemComponent";
import {
  ActionRequest,
  ActionReply,
  ResourceHealth,
  ResourceStatus,
  SyncResourceRequest,
  SyncResourceReply,
  CalculatePropertiesRequest,
  CalculatePropertiesResult,
} from "../../veritech/intelligence";
import { Event, EventLogLevel } from "../../veritech/eventLog";
import { siExec } from "../siExec";
import { promises as fs } from "fs";
import os from "os";
import path from "path";

const intelligence = (registry.get("dockerImage") as EntityObject).intelligence;

intelligence.calculateProperties = function(
  req: CalculatePropertiesRequest,
): CalculatePropertiesResult {
  const result: CalculatePropertiesResult = {
    inferredProperties: {
      __baseline: {},
    },
  };
  for (const pred of req.predecessors) {
    if (pred.entity.objectType == "service") {
      result.inferredProperties.__baseline["image"] = pred.entity.name;
    }
  }
  return result;
};

intelligence.syncResource = async function(
  request: SyncResourceRequest,
  event: Event,
): Promise<SyncResourceReply> {
  const state: Record<string, any> = {};

  let tempdir;
  for (const pred of request.predecessors) {
    if (pred.entity.objectType == "dockerHubCredential") {
      const logEntry = event.log(
        EventLogLevel.Debug,
        "using docker hub credential",
        { name: pred.entity.name },
      );
      try {
        tempdir = await fs.mkdtemp(path.join(os.tmpdir(), "docker-"));
        const creds = pred.entity.properties.__baseline.decrypted;
        console.dir({ creds });
        const auth = Buffer.from(
          `${creds?.username}:${creds?.password}`,
        ).toString("base64");
        const config = {
          auths: {
            "https://index.docker.io/v1/": {
              auth,
            },
          },
        };
        await fs.writeFile(
          path.join(tempdir, "config.json"),
          JSON.stringify(config, null, 0),
          { mode: 0o400 },
        );
      } catch (e) {
        logEntry.payload["failure"] = `${e}`;
        logEntry.fatal();
      }
    }
  }

  // Pull the image with `docker image pull`
  let pullArgs: string[] = [];
  if (tempdir) {
    pullArgs = pullArgs.concat(["--config", tempdir]);
  }
  pullArgs = pullArgs.concat([
    "image",
    "pull",
    request.entity.properties.__baseline.image,
  ]);
  console.log(`running command; cmd="docker ${pullArgs.join(" ")}"`);
  const dockerImagePull = await siExec(event, "docker", pullArgs, {
    reject: false,
  });

  // If the image pull failed, early return
  if (dockerImagePull.failed) {
    const state = {
      data: request.resource.state?.data,
      errorMsg: "docker image pull command failed",
      errorOutput: dockerImagePull.stderr,
    };

    return {
      resource: {
        state,
        health: ResourceHealth.Error,
        status: ResourceStatus.Failed,
      },
    };
  }

  // Inspect the now-local image
  let inspectArgs: string[] = [];
  if (tempdir) {
    inspectArgs = inspectArgs.concat(["--config", tempdir]);
  }
  inspectArgs = inspectArgs.concat([
    "image",
    "inspect",
    request.entity.properties.__baseline.image,
  ]);
  console.log(`running command; cmd="docker ${inspectArgs.join(" ")}"`);
  const dockerImageInspect = await siExec(event, "docker", inspectArgs, {
    reject: false,
  });

  let health: ResourceHealth = ResourceHealth.Ok;
  if (dockerImageInspect.failed) {
    health = ResourceHealth.Error;
    state["data"] = request.resource.state;
    state["errorMsg"] = dockerImageInspect.stderr;

    return {
      resource: {
        state,
        health,
        status: ResourceStatus.Failed,
      },
    };
  }

  // Set state data
  const dockerImageJson = JSON.parse(dockerImageInspect.stdout);
  state["data"] = dockerImageJson[0];

  return {
    resource: {
      state,
      health,
      status: ResourceStatus.Created,
    },
  };
};

intelligence.actions = {
  async deploy(request: ActionRequest): Promise<ActionReply> {
    const actions: ActionReply["actions"] = [];
    for (const child of request.successors) {
      if (child.entity.objectType == "service") {
        actions.push({ action: "deploy", entityId: child.entity.id });
      }
    }
    const reply: ActionReply = {
      resource: {
        state: {
          alex: "van halen",
          deployedBy: request.predecessors.map(p => p.entity.name),
        },
        health: ResourceHealth.Ok,
        status: ResourceStatus.Created,
      },
      actions,
    };
    return reply;
  },
};

use httpmock::prelude::*;
use seaplane::api::compute::v2::Formation as FormationModel;
use seaplane_cli::{
    context::Ctx, ops::formation::Formation, printer::printer, test_main_exec_with_ctx,
};
use serde_json::json;
use wildmatch::WildMatch;

use super::{then, when, when_json, MOCK_SERVER};

// The ARGV must use the name `stubb` as these are just tests, no need to spend brainpower trying to
// figure out how to make it perfectly generic.
//
// Similarly we must use the long forms of `--fetch` and `--force`, etc.
//

const DEFAULT_CFG_UUID: &str = "46c5d58c-7b8b-4e8d-9e98-26bb31b9ab8f";

fn build_ctx_with_default_formation(local_only: bool) -> Ctx {
    let fcm: FormationModel = serde_json::from_str(&default_cfg_json().to_string()).unwrap();
    let fc = Formation::new(fcm);
    let mut f = Formation::new("stubb");
    f.local.insert(fc.id);
    if !local_only {
        f.in_air.insert(fc.id);
    }

    let mut ctx = Ctx::default();
    ctx.compute_url = Some(MOCK_SERVER.base_url().parse().unwrap());
    ctx.identity_url = Some(MOCK_SERVER.base_url().parse().unwrap());
    ctx.db.formations.formations.push(f);
    ctx.db.formations.configurations.push(fc);
    ctx
}

fn default_cfg_json() -> serde_json::Value {
    json!({
        "flights":[{
            "name":"flask",
            "image":"registry.cplane.cloud/stubb/alpine:latest",
        },
        {
            "name":"pequod",
            "image":"registry.cplane.cloud/stubb/alpine:latest",
        }]
    })
}

// This could be a fn and not a macro...but as a macro we don't have to worry about return values
// and other things. And since this is just a test the macro is easier.
//
// For cfg_uuids_json only a single UUID can be passed in.
macro_rules! mock_fetch {
    (@impl $name:expr, $cfg_uuids_json:expr, $active_cfgs_json:expr, $cfg_json:expr) => {{
        let mut name = $name;
        let uuid = $cfg_uuids_json.as_array()
            .unwrap()
            .get(0)
            .map(ToOwned::to_owned)
            .unwrap_or(serde_json::Value::String(String::from("this-should-not-happen")))
            .as_str()
            .unwrap()
            .to_string();

        // Step 0: Get a list of formation names
        let step_0_resp_body = if $name.is_empty() {
            name = "this-should-not-happen".into();
            json!([])
        } else {
            json!([$name])
        };
        let list_names = MOCK_SERVER.mock(|w, then| {
            when(w, GET, "/v1/formations");
            then.status(200).json_body_obj(&step_0_resp_body);
        });

        // Step 1: Get a list of CFG UUIDs
        let list_cfg_uuids = MOCK_SERVER.mock(|w, then| {
            when_json(w, GET, format!("/v1/formations/{name}/configurations"));
            then.status(200).json_body($cfg_uuids_json);
        });

        // Step 2: Get a list of Active CFG UUIDs
        let list_active_cfg_uuids = MOCK_SERVER.mock(|w, then| {
            when_json(w, GET, format!("/v1/formations/{name}/activeConfiguration"));
            then.status(200).json_body($active_cfgs_json);
        });

        // Step 3: Get the configuration for UUID ...
        let get_cfg = MOCK_SERVER.mock(|w, then| {
            when_json(
                w,
                GET,
                format!("/v1/formations/{name}/configurations/{uuid}"),
            );
            then.status(200).json_body($cfg_json);
        });

        (vec![list_names], vec![list_cfg_uuids], vec![list_active_cfg_uuids], vec![get_cfg])
    }};
    (@extra $name:expr, $extra_uuid:expr, $cfg_json:expr) => {{
        let (mut a1, mut b1, mut c1, mut d1) = mock_fetch!(@impl
            $name,
            json!([$extra_uuid]),
            json!([{
                "configuration_id" : $extra_uuid,
                "traffic_weight" : 1.0_f32
            }]),
            $cfg_json
        );
        let (a2, b2, c2, d2) = mock_fetch!(@impl "stubb", json!([]), json!([]), json!({}));
        a1.extend(a2); b1.extend(b2); c1.extend(c2); d1.extend(d2);
        (a1, b1, c1, d1)
    }};
    (@no_remote_instances) => {{
        mock_fetch!(@impl "", json!([]), json!([]), json!({}))
    }};
    () => {{
        mock_fetch!(@impl
            "stubb",
            json!([DEFAULT_CFG_UUID]),
            json!([{
                "configuration_id" : DEFAULT_CFG_UUID,
                "traffic_weight" : 1.0_f32
            }]),
            default_cfg_json()
        )
    }};
}

macro_rules! test_fn_land {
    ($test_fn:ident, $argv:expr, remote_instances = $remote_instances:expr) => {
        #[test]
        fn $test_fn() {
            let fetch = $argv.contains("--fetch");
            let mut land_formation = MOCK_SERVER.mock(|w, then| {
                when(w, DELETE, "/v1/formations/stubb/activeConfiguration");
                then.status(200).body("success");
            });

            let (
                mut list_names,
                mut list_cfg_uuids,
                mut list_active_cfg_uuids,
                mut get_cfg
            ) = if $remote_instances {
                mock_fetch!()
            } else {
                mock_fetch!(@no_remote_instances)
            };

            let mut ctx = Ctx::default();
            ctx.compute_url = Some(MOCK_SERVER.base_url().parse().unwrap());
            ctx.identity_url = Some(MOCK_SERVER.base_url().parse().unwrap());
            ctx.db.formations.formations.push(Formation::new("stubb"));
            let res = test_main_exec_with_ctx(&argv!($argv), ctx);
            assert!(res.is_ok(), "{res:?}");

            assert_eq!(land_formation.hits(), 1, "land_formation");
            assert_eq!(list_names[0].hits(), if fetch { 1 } else { 0 }, "list_names");
            assert_eq!(list_cfg_uuids[0].hits(), if fetch && $remote_instances { 1 } else { 0 }, "list_cfg_uuids");
            assert_eq!(list_active_cfg_uuids[0].hits(), if fetch && $remote_instances { 1 } else { 0 }, "list_active_cfg_uuids");
            assert_eq!(get_cfg[0].hits(), if fetch && $remote_instances { 1 } else { 0 }, "get_cfg");

            let correct_out= "Successfully Landed remote Formation Instance 'stubb'";

            let actual_out: String = printer().as_string().trim().to_string();
            assert!(WildMatch::new(&correct_out).matches(&actual_out), "{actual_out}");
            printer().clear();
            land_formation.delete();
            list_cfg_uuids.iter_mut().for_each(|m| m.delete());
            list_names.iter_mut().for_each(|m| m.delete());
            list_active_cfg_uuids.iter_mut().for_each(|m| m.delete());
            get_cfg.iter_mut().for_each(|m| m.delete());
        }
    };
}

test_fn_land!(formation_land, "formation land stubb", remote_instances = false);
test_fn_land!(formation_land_fetch, "formation land stubb --fetch", remote_instances = false);
test_fn_land!(
    formation_land_fetch_w_remotes,
    "formation land stubb --fetch",
    remote_instances = true
);

macro_rules! test_fn_delete {
    ($test_fn:ident, $argv:expr, remote_instances = $remote_instances:expr) => {
        #[test]
        fn $test_fn() {
            let force = $argv.contains("--force");
            let fetch = $argv.contains("--fetch");
            let resp_json = json!([DEFAULT_CFG_UUID]);

            let mut delete_formation = MOCK_SERVER.mock(|w, t| {
                when_json(w, DELETE, "/v1/formations/stubb")
                    .query_param("force", format!("{}", force));
                then(t, &resp_json);
            });

            let (
                mut list_names,
                mut list_cfg_uuids,
                mut list_active_cfg_uuids,
                mut get_cfg
            ) = if $remote_instances {
                mock_fetch!()
            } else {
                mock_fetch!(@no_remote_instances)
            };

            let mut ctx = Ctx::default();
            ctx.compute_url = Some(MOCK_SERVER.base_url().parse().unwrap());
            ctx.identity_url = Some(MOCK_SERVER.base_url().parse().unwrap());
            ctx.db.formations.formations.push(Formation::new("stubb"));
            let res = test_main_exec_with_ctx(&argv!($argv), ctx);
            assert!(res.is_ok(), "{res:?}");

            assert_eq!(delete_formation.hits(), 1, "delete_formation");
            assert_eq!(list_names[0].hits(), if fetch { 1 } else { 0 }, "list_names");
            assert_eq!(list_cfg_uuids[0].hits(), if fetch && $remote_instances { 1 } else { 0 }, "list_cfg_uuids");
            assert_eq!(list_active_cfg_uuids[0].hits(), if fetch && $remote_instances { 1 } else { 0 }, "list_active_cfg_uuids");
            assert_eq!(get_cfg[0].hits(), if fetch && $remote_instances { 1 } else { 0 }, "get_cfg");

            let correct_out: String = format!("Deleted remote Formation Instance 'stubb' with Configuration UUIDs:\n\t\
                {DEFAULT_CFG_UUID}\n\n\
                Successfully removed 1 item");

            let actual_out: String = printer().as_string().trim().to_string();
            assert!(WildMatch::new(&correct_out).matches(&actual_out), "{actual_out}");
            printer().clear();
            delete_formation.delete();
            list_cfg_uuids.iter_mut().for_each(|m| m.delete());
            list_names.iter_mut().for_each(|m| m.delete());
            list_active_cfg_uuids.iter_mut().for_each(|m| m.delete());
            get_cfg.iter_mut().for_each(|m| m.delete());
        }
    };
}

test_fn_delete!(formation_del, "formation delete stubb", remote_instances = false);
test_fn_delete!(formation_del_force, "formation delete stubb --force", remote_instances = false);
test_fn_delete!(formation_del_fetch, "formation delete stubb --fetch", remote_instances = false);
test_fn_delete!(
    formation_del_fetch_w_remotes,
    "formation delete stubb --fetch",
    remote_instances = true
);
test_fn_delete!(
    formation_del_fetch_force_wo_remotes,
    "formation delete stubb --fetch --force",
    remote_instances = false
);
test_fn_delete!(
    formation_del_fetch_force_w_remotes,
    "formation delete stubb --fetch --force",
    remote_instances = true
);

macro_rules! mock_launch {
    (@impl
         $expected_json:expr,
         $argv:expr,
         $ctx:expr,
         should_create=$should_create:expr,
         remote_instances=$remote_instances:expr
    ) => {{
        let grounded = $argv.contains("--grounded");
        let fetch = $argv.contains("--fetch");
        let inline_flight = $argv.contains("--include-flight-plan");
        let launch = $argv.contains(" launch");

        let (
            mut list_names,
            mut list_cfg_uuids,
            mut list_active_cfg_uuids,
            mut get_cfg
        ) = if $remote_instances && $should_create {
            // The default UUIDs and Configs collide with what we're trying to "launch" so
            // we make up some random new ones just to simulate, "We can fetch something unrelated
            // an launch this new thing too"
            mock_fetch!(@extra
                "queequeg",
                "117f87c3-c26c-228c-c970-cb8acac2bd11",
                json!({
                    "flights":[{
                        "name":"ishmael",
                        "image":"registry.cplane.cloud/queequeg/alpine:latest",
                    }],
                })
            )
        } else if $remote_instances && !$should_create {
            // This simulates we gave a command like `launch FOO --fetch` but FOO does not exist on
            // our local machine, yet it does exist in the Seaplane cloud (presumably grounded)
            mock_fetch!(@impl
                "stubb",
                json!([DEFAULT_CFG_UUID]),
                json!([]),
                default_cfg_json()
            )
        } else {
            // In this scenario we've passed `--fetch` but there is nothing in the Seaplane cloud
            // to fetch
            mock_fetch!(@no_remote_instances)
        };

        // Step 1: It tries to add a configuration to an existing formation, but none exists
        let step1_status = if $should_create { 404 } else { 201 };
        let mut try_to_add_cfg = MOCK_SERVER.mock(|w, then| {
            when_json(w, POST, "/v1/formations/stubb/configurations")
                .query_param("active", "false")
                .json_body_obj(&$expected_json);
            if $should_create {
                then.status(step1_status);
            } else {
                then.status(step1_status).json_body_obj(&json!(DEFAULT_CFG_UUID));
            };
        });

        // Step 2: It tries to create a new formation and succeeds
        let step_2_resp_body = json!([DEFAULT_CFG_UUID]);
        let mut create_new_formation = MOCK_SERVER.mock(|w, then| {
            when_json(w, POST, "/v1/formations/stubb")
                .query_param("active", format!("{:?}", !grounded))
                .json_body_obj(&$expected_json);
            then.status(201)
                .json_body_obj(&step_2_resp_body);
        });

        // Step 3: Request subdomain from metadata
        let step_3_resp_body = json!({"url":"https://stubb--bar.on.cplane.cloud/"});
        let mut get_subdomain = MOCK_SERVER.mock(|w, t| {
            when_json(w, GET, "/v1/formations/stubb");
            then(t, &step_3_resp_body);
        });

        // Step 5: Set a configuration to active
        let mut set_cfg_to_active = MOCK_SERVER.mock(|w, then| {
            when_json(w, PUT, "/v1/formations/stubb/activeConfiguration")
                .query_param("force", "false")
                .json_body_obj(&json!([{"configuration_id":DEFAULT_CFG_UUID, "traffic_weight":1.0_f32}]));
            then.status(200);
        });

        let res = test_main_exec_with_ctx(&argv!($argv), $ctx);
        assert!(res.is_ok(), "{} {res:?}", $argv);

        // Figuring out endpoints that should have been hit is somewhat tough. It depends on if we
        // did a --fetch, and if the there were remote instances or not (and what those remote
        // instances were). In some cases, the remote instances simulated, "We created this
        // somewhere else, but on this machine we want to launch it" while other times it just
        // simulates a "Seaplane knows about about other formations, we're fetching but nothing
        // should conflict"
        //
        // These cases are tracked in `$should_create` and `$remote_instances`
        //
        // If --fetch was not used, we can assume $remote_instances is false (i.e. we're not
        // fetching anything remote).
        //
        // $should_create means we should be hitting the "create_new_formation" endpoints
        //
        // $remote_instances is dual purposed and the reason why the mock objects returned from
        // mock_fetch! are a Vec and not a single object.
        //
        // $remote_instances can either be a remote instance of *this* formation to launch (when
        // $should_create=false) OR just an unrelated remote instance that doesn't conflict with
        // anything we're doing.
        if fetch {
            // There could be multiple mock objects for this common endpoint, sooo....at least one
            // of this must be hit but we don't care which
            assert!(list_names.iter().any(|m| m.hits() > 0), "list_names endpoint not hit");

            if $should_create && $remote_instances {
                // In this branch the "unrelated" formation is being fetched (queequeg), but
                // *this* formation (stubb) is only local
                let queequeg_lcu = &list_cfg_uuids[0];
                let stubb_lcu = &list_cfg_uuids[1];
                assert_eq!(queequeg_lcu.hits(), 1, "queequeg_lcu");
                assert_eq!(stubb_lcu.hits(), 0, "stubb_lcu");

                let queequeg_acu = &list_active_cfg_uuids[0];
                let stubb_acu = &list_active_cfg_uuids[1];
                assert_eq!(queequeg_acu.hits(), 1, "queequeg_acu");
                assert_eq!(stubb_acu.hits(), 0, "stubb_acu");

                let queequeg_gc = &get_cfg[0];
                let stubb_gc = &get_cfg[1];
                assert_eq!(queequeg_gc.hits(), 1, "queequeg_gc");
                assert_eq!(stubb_gc.hits(), 0, "stubb_gc");
            } else if $remote_instances {
                assert_eq!(list_cfg_uuids[0].hits(), if grounded { 1 } else {2},"list_cfg_uuids");
                assert_eq!(list_active_cfg_uuids[0].hits(), 1, "list_active_cfg_uuids");
                assert_eq!(get_cfg[0].hits(), 1, "get_cfg");
            }
        } else {
            // We didn't do a --fetch
            assert_eq!(list_names[0].hits(), 0, "list_names");
            assert_eq!(list_cfg_uuids[0].hits(), 0, "list_cfg_uuids");
            assert_eq!(list_active_cfg_uuids[0].hits(), 0, "list_active_cfg_uuids");
            assert_eq!(get_cfg[0].hits(), 0, "get_cfg");
        }

        if $should_create && $remote_instances {
            assert_eq!(create_new_formation.hits(), 1, "create_new_formation");
            assert_eq!(set_cfg_to_active.hits(), 0, "set_cfg_to_active");
        } else if $remote_instances {
            assert_eq!(create_new_formation.hits(), 0, "create_new_formation");
            assert_eq!(set_cfg_to_active.hits(), if grounded { 0 } else { 1 }, "set_cfg_to_active");
        }
        assert_eq!(try_to_add_cfg.hits(), if grounded && !$should_create { 0 } else { 1 }, "try_to_add_cfg");
        assert_eq!(get_subdomain.hits(), if grounded && !$should_create { 0 } else {1}, "get_subdomain");

        let fetched_other_correct_out: &str = r#"Successfully synchronized Formation Instance 'queequeg' with local Formation ID '????????'
Successfully synchronized Flight Plan 'ishmael' with local Flight Plan ID '????????'!

Successfully fetched 2 items"#;
        let fetched_self_correct_out: &str = r#"Successfully synchronized Formation Instance 'stubb' with local Formation ID '????????'
Successfully synchronized Flight Plan '*' with local Flight Plan ID '????????'!
Successfully synchronized Flight Plan '*' with local Flight Plan ID '????????'!

Successfully fetched 3 items"#;
        let up_to_date_correct_out: &str = "All local definitions are up to date!";
        let grounded_correct_out: &str = "Formation Plan 'stubb' is already uploaded and in status Grounded";
        let inline_correct_out: &str = r#"Successfully created Flight Plan 'flask' with ID '????????'
Successfully created Flight Plan 'pequod' with ID '????????'
Successfully created local Formation Plan 'stubb' with ID '????????'"#;
        let launched_correct_out: String = r#"Successfully Launched remote Formation Instance 'stubb' with Configuration UUIDs:
????????-????-????-????-????????????
The remote Formation Instance URL is https://stubb--bar.on.cplane.cloud/
(hint: it may take up to a minute for the Formation to become fully online)
(hint: check the status of this Formation Instance with 'seaplane formation status stubb')"#.into();
        let mut correct_out = String::new();
        if fetch && !inline_flight && !launch {
            if !$should_create {
                correct_out = format!("{fetched_self_correct_out}");
            } else if $remote_instances {
                correct_out = format!("{fetched_other_correct_out}");
            } else {
                correct_out = format!("{up_to_date_correct_out}");
            }
        }
        if inline_flight {
            if correct_out.is_empty() {
                correct_out = format!("{inline_correct_out}");
            } else {
                correct_out = format!("{correct_out}\n{inline_correct_out}");
            }
        }
        if grounded {
            if correct_out.is_empty() {
                if $remote_instances && !$should_create {
                    correct_out = format!("{grounded_correct_out}");
                } else {
                    correct_out = format!("{launched_correct_out}");
                }
            } else {
                if $remote_instances && !$should_create {
                    correct_out = format!("{correct_out}\n{grounded_correct_out}");
                } else {
                    correct_out = format!("{correct_out}\n{launched_correct_out}");
                }
            }
        } else {
            if correct_out.is_empty() {
                correct_out = format!("{launched_correct_out}");
            } else {
                correct_out = format!("{correct_out}\n{launched_correct_out}");
            }
        }

        let actual_out: String = printer().as_string().trim().to_string();
        assert!(WildMatch::new(&correct_out).matches(&actual_out), "\ncorrect:\n{correct_out}\n\nactual:\n{actual_out}");
        printer().clear();
        try_to_add_cfg.delete();
        create_new_formation.delete();
        get_subdomain.delete();
        set_cfg_to_active.delete();
        list_names.iter_mut().for_each(|m| m.delete());
        list_cfg_uuids.iter_mut().for_each(|m| m.delete());
        list_active_cfg_uuids.iter_mut().for_each(|m| m.delete());
        get_cfg.iter_mut().for_each(|m| m.delete());
    }};
    (
        $argv:expr,
        should_create = $should_create:expr,
        remote_instances = $remote_instances:expr,
        expected_json = $expected_json:expr
    ) => {{
        let mut ctx = Ctx::default();
        ctx.compute_url = Some(MOCK_SERVER.base_url().parse().unwrap());
        ctx.identity_url = Some(MOCK_SERVER.base_url().parse().unwrap());
        mock_launch!(@impl
            $expected_json,
            $argv,
            ctx,
            should_create=$should_create,
            remote_instances = $remote_instances
        );
    }};
    (
        $argv:expr,
        $ctx:expr,
        should_create = $should_create:expr,
        remote_instances = $remote_instances:expr
    ) => {{
        if $ctx.compute_url.is_none() {
            $ctx.compute_url = Some(MOCK_SERVER.base_url().parse().unwrap());
        }
        if $ctx.identity_url.is_none() {
            $ctx.identity_url = Some(MOCK_SERVER.base_url().parse().unwrap());
        }
        mock_launch!(@impl
            default_cfg_json(),
            $argv,
            $ctx,
            should_create=$should_create,
            remote_instances = $remote_instances
        );
    }};
    (
        $argv:expr,
        should_create = $should_create:expr,
        remote_instances = $remote_instances:expr
    ) => {{
        mock_launch!(
            $argv,
            should_create=$should_create,
            remote_instances = $remote_instances,
            expected_json = default_cfg_json()
        );
    }};
}

#[test]
fn formation_plan_launch() {
    mock_launch!(
        "formation plan \
            --name stubb \
            --include-flight-plan name=pequod,image=stubb/alpine:latest \
            --include-flight-plan name=flask,image=stubb/alpine:latest \
            --public-endpoint /=flask:80 \
            --launch",
        should_create = true,
        remote_instances = false
    );
}

#[cfg_attr(feature = "unstable", test)]
#[cfg_attr(not(feature = "unstable"), allow(dead_code))]
fn formation_plan_launch_all_fields_unstable() {
    mock_launch!(
        "formation plan \
            --name stubb \
            --include-flight-plan name=pequod,image=stubb/alpine:latest \
            --include-flight-plan name=flask,image=stubb/alpine:latest,min=5,max=20,architecture=amd64,api-permission \
            --launch",
        should_create = true,
        remote_instances = false,
        expected_json = json!({
            "flights":[{
                "name":"flask",
                "image":"registry.cplane.cloud/stubb/alpine:latest",
            },{
                "name":"pequod",
                "image":"registry.cplane.cloud/stubb/alpine:latest",
            }],
        })
    );
}

#[test]
fn formation_plan_launch_all_fields() {
    mock_launch!(
        "formation plan \
            --name stubb \
            --include-flight-plan name=pequod,image=stubb/alpine:latest \
            --include-flight-plan name=flask,image=stubb/alpine:latest,min=5,max=20,architecture=amd64 \
            --providers aws \
            --exclude-providers azure \
            --regions xu \
            --exclude-regions xn \
            --public-endpoint /=flask:80,/foo=pequod:8443 \
            --flight-endpoint udp:2424=pequod:9090,tcp:2222=flask:22 \
            --launch",
        should_create = true,
        remote_instances = false,
        expected_json = json!({
            "flights":[{
                "name":"flask",
                "image":"registry.cplane.cloud/stubb/alpine:latest",
            },{
                "name":"pequod",
                "image":"registry.cplane.cloud/stubb/alpine:latest",
            }],
        })
    );
}

#[test]
fn formation_plan_launch_fetch() {
    mock_launch!(
        "formation plan \
                --name stubb \
                --include-flight-plan name=pequod,image=stubb/alpine:latest \
                --include-flight-plan name=flask,image=stubb/alpine:latest \
                --public-endpoint /=flask:80 \
                --launch \
                --fetch",
        should_create = true,
        remote_instances = false
    );
}

#[test]
fn formation_plan_grounded_new() {
    mock_launch!(
        "formation plan \
            --name stubb \
            --include-flight-plan name=pequod,image=stubb/alpine:latest \
            --include-flight-plan name=flask,image=stubb/alpine:latest \
            --public-endpoint /=flask:80 \
            --grounded",
        should_create = true,
        remote_instances = false
    );
}

#[test]
fn formation_launch() {
    let mut ctx = build_ctx_with_default_formation(true);
    mock_launch!("formation launch stubb", ctx, should_create = true, remote_instances = false);
}

#[test]
fn formation_launch_fetch() {
    for b in [true, false] {
        let mut ctx = build_ctx_with_default_formation(true);
        mock_launch!(
            "formation launch stubb --fetch",
            ctx,
            should_create = true,
            remote_instances = b
        );
    }
    mock_launch!("formation launch stubb --fetch", should_create = false, remote_instances = true);
}

#[test]
fn formation_launch_grounded() {
    let mut ctx = build_ctx_with_default_formation(true);
    mock_launch!(
        "formation launch stubb --grounded",
        ctx,
        should_create = true,
        remote_instances = false
    );
}

#[test]
fn formation_launch_grounded_fetch() {
    for b in [true, false] {
        let mut ctx = build_ctx_with_default_formation(true);
        mock_launch!(
            "formation launch stubb --grounded --fetch",
            ctx,
            should_create = true,
            remote_instances = b
        );
    }
    mock_launch!(
        "formation launch stubb --grounded --fetch",
        should_create = false,
        remote_instances = true
    );
}

macro_rules! test_fn_fetch {
    ($test_fn:ident, $argv:expr) => {
        #[test]
        fn $test_fn() {
            // "formation fetch-remote" == 2; get all formations
            // "formation fetch-remote stubb" == 3; get just the stubb formation
            let is_all = $argv.split(' ').count() == 2;

            let (
                mut list_names,
                mut list_cfg_uuids,
                mut list_active_cfg_uuids,
                mut get_cfg
            ) = mock_fetch!();

            let ctx = build_ctx_with_default_formation(false);
            let res = test_main_exec_with_ctx(&argv!($argv), ctx);
            assert!(res.is_ok(), "{res:?}");
            assert_eq!(list_names[0].hits(), if is_all {1} else {0}, "list_names");
            assert_eq!(list_cfg_uuids[0].hits(), 1, "list_cfg_uuids");
            assert_eq!(list_active_cfg_uuids[0].hits(), 1, "list_active_cfg_uuids");
            assert_eq!(get_cfg[0].hits(), 1, "get_cfg");
            let correct_out = r#"Successfully synchronized Formation Configuration in Formation 'stubb' with local Formation Configuration ID '????????'
Successfully synchronized Flight Plan '*' with local Flight Plan ID '????????'!
Successfully synchronized Flight Plan '*' with local Flight Plan ID '????????'!

Successfully fetched 3 items"#;
            let actual_out: String = printer().as_string().trim().to_string();
            assert!(
                WildMatch::new(&correct_out).matches(&actual_out),
                "{actual_out}"
            );
            printer().clear();
            list_names.iter_mut().for_each(|m| m.delete());
            list_cfg_uuids.iter_mut().for_each(|m| m.delete());
            list_active_cfg_uuids.iter_mut().for_each(|m| m.delete());
            get_cfg.iter_mut().for_each(|m| m.delete());
        }
    };
}

test_fn_fetch!(formation_fetch_one, "formation fetch-remote stubb");
test_fn_fetch!(formation_fetch_all, "formation fetch-remote");

macro_rules! test_fn_status {
    (@impl $argv:expr, $ctx:expr) => {{
        // "formation status" == 2; get all formations
        // "formation status stubb" == 3; get just the stubb formation
        let is_all = $argv.split(' ').count() == 2;
        let fetch = !$argv.contains("--no-fetch");

        let (
            mut list_names,
            mut list_cfg_uuids,
            mut list_active_cfg_uuids,
            mut get_cfg
        ) = mock_fetch!(@impl
            "stubb",
            json!([DEFAULT_CFG_UUID]),
            json!([{
                "configuration_id" : DEFAULT_CFG_UUID,
                "traffic_weight" : 1.0_f32
            }]),
            default_cfg_json()
            );

        let get_containers_resp_body = json!(
            [{
                "container_id" : "557f87c3-b26c-428c-b970-cb8acac2bd68",
                "status" : "started",
                "flight_name": "flask",
                "configuration_id" : DEFAULT_CFG_UUID,
                "exit_status": 0_i32,
                "start_time": "2022-04-26 10:23:09Z",
                "stop_time": "2022-04-26 10:25:09Z",
                "public_ingress_usage": 123456_u64,
                "public_egress_usage": 123456_u64,
                "private_ingress_usage": 123456_u64,
                "private_egress_usage": 123456_u64,
                "disk_usage": 123456_u64,
                "ram_usage": 123456_u64,
                "cpu_usage": 123456_u64,
                "host_latitude": 29.984142_f32,
                "host_longitude": -95.332986_f32,
                "host_iata": "IAH",
                "host_country": "US",
                "host_region": Region::XN,
                "host_provider": Provider::AWS,
            },
            {
                "container_id" : "91f191f5-be32-4d44-860f-0eccca325e0f",
                "status" : "running",
                "flight_name": "pequod",
                "configuration_id" : "46c5d58c-7b8b-4e8d-9e98-26bb31b9ab8f",
                "exit_status": 1_i32,
                "start_time": "2022-04-26 10:23:09Z",
                "stop_time": "2022-04-26 10:25:09Z",
                "public_ingress_usage": 123456_u64,
                "public_egress_usage": 123456_u64,
                "private_ingress_usage": 123456_u64,
                "private_egress_usage": 123456_u64,
                "disk_usage": 123456_u64,
                "ram_usage": 123456_u64,
                "cpu_usage": 123456_u64,
                "host_latitude": 29.984142_f32,
                "host_longitude": -95.332986_f32,
                "host_iata": "IAH",
                "host_country": "US",
                "host_region": Region::XN,
                "host_provider": Provider::AWS,
            }]
        );

        let mut get_containers = MOCK_SERVER.mock(|w, then| {
            when_json(w, GET, "/v1/formations/stubb/containers");
            then.status(200).json_body_obj(&get_containers_resp_body);
        });

        let res = test_main_exec_with_ctx(&argv!($argv), $ctx);
        assert!(res.is_ok(), "{res:?}");

        assert_eq!(list_names[0].hits(), if is_all || fetch { 1 } else { 0 }, "list_names");
        assert_eq!(list_cfg_uuids[0].hits(), if fetch { 1 } else { 0 }, "list_cfg_uuids");
        assert_eq!(list_active_cfg_uuids[0].hits(), if fetch {1} else {0}, "list_active_cfg_uuids");
        assert_eq!(get_cfg[0].hits(), if fetch {1} else {0}, "get_cfg");
        assert_eq!(get_containers.hits(), 1, "get_containers");

        let correct_out = r#"◉ Formation stubb: DEGRADED
│
└─◉ Configuration 46c5d58c-7b8b-4e8d-9e98-26bb31b9ab8f: DEGRADED
  │
  │   FLIGHT    RUNNING    EXITED    ERRORED    STARTING    MIN / MAX
  ├─◉ flask     0          0         0          1           1 / AUTO
  └─◉ pequod    1          0         0          0           1 / AUTO"#;
        let actual_out: String = printer().as_string().trim().to_string();
        assert!(
            WildMatch::new(&correct_out).matches(&actual_out),
            "\ncorrect:\n{correct_out}\n\nactual:\n{actual_out}"
        );
        printer().clear();
        list_names.iter_mut().for_each(|m| m.delete());
        list_cfg_uuids.iter_mut().for_each(|m| m.delete());
        list_active_cfg_uuids.iter_mut().for_each(|m| m.delete());
        get_cfg.iter_mut().for_each(|m| m.delete());
        get_containers.delete();
    }};
    ($argv:expr, $ctx:expr) => {{
        test_fn_status!(@impl $argv, $ctx);
    }};
    ($argv:expr) => {{
        test_fn_status!(@impl $argv, build_ctx_with_default_formation(false));
    }};
}

#[test]
fn formation_status_all_fetch() {
    test_fn_status!("formation status");
}

#[test]
fn formation_status_one_fetch() {
    test_fn_status!("formation status stubb");
}

fn ctx_with_remote_id() -> Ctx {
    let fcm: FormationModel = serde_json::from_str(&default_cfg_json().to_string()).unwrap();
    let mut fc = Formation::new(fcm);
    fc.remote_id = Some(DEFAULT_CFG_UUID.parse().unwrap());
    let mut f = Formation::new("stubb");
    f.local.insert(fc.id);
    f.in_air.insert(fc.id);

    let mut ctx = Ctx::default();
    ctx.compute_url = Some(MOCK_SERVER.base_url().parse().unwrap());
    ctx.identity_url = Some(MOCK_SERVER.base_url().parse().unwrap());
    ctx.db.formations.formations.push(f);
    ctx.db.formations.configurations.push(fc);
    ctx
}

#[test]
fn formation_status_all_no_fetch() {
    test_fn_status!("formation status --no-fetch", ctx_with_remote_id());
}

#[test]
fn formation_status_one_no_fetch() {
    test_fn_status!("formation status stubb --no-fetch", ctx_with_remote_id());
}

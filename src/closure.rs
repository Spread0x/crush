use crate::job::Job;
use crate::namepspace::Namespace;
use crate::data::Argument;
use crate::stream::empty_channel;
use crate::errors::{error, CrushResult, mandate};
use crate::lib::{ExecutionContext, StreamExecutionContext};
use crate::stream_printer::spawn_print_thread;

#[derive(Clone)]
#[derive(Debug)]
pub struct Closure {
    job_definitions: Vec<Job>,
    env: Option<Namespace>,
}

impl Closure {
    pub fn new(job_definitions: Vec<Job>) -> Closure {
        Closure {
            job_definitions,
            env: None,
        }
    }

    pub fn with_env(&self, env: &Namespace) -> Closure {
        Closure {
            job_definitions: self.job_definitions.clone(),
            env: Some(env.clone()),
        }
    }

    pub fn spawn_stream(&self, context: StreamExecutionContext) -> CrushResult<()> {
        let job_definitions = self.job_definitions.clone();
        let parent_env = mandate(self.env.clone(), "Closure without env")?;
        Ok(())
    }

    pub fn spawn_and_execute(&self, context: ExecutionContext) -> CrushResult<()> {
        let job_definitions = self.job_definitions.clone();
        let parent_env = mandate(self.env.clone(), "Closure without env")?;
        let env = parent_env.create_child();

        Closure::push_arguments_to_env(context.arguments, &env);
        match job_definitions.len() {
            0 => return error("Empty closures not supported"),
            1 => {
                let job = job_definitions[0].spawn_and_execute(&env, &context.printer, context.input, context.output)?;
                job.join(&context.printer);
            }
            _ => {
                let first_job_definition = &job_definitions[0];
                let last_output = spawn_print_thread(&context.printer);
                let first_job = first_job_definition.spawn_and_execute(&env, &context.printer, context.input, last_output)?;
                first_job.join(&context.printer);

                for job_definition in &job_definitions[1..job_definitions.len() - 1] {
                    let last_output = spawn_print_thread(&context.printer);
                    let job = job_definition.spawn_and_execute(&env, &context.printer, empty_channel(), last_output)?;
                    job.join(&context.printer);
                }

                let last_job_definition = &job_definitions[job_definitions.len() - 1];
                let last_job = last_job_definition.spawn_and_execute(&env, &context.printer, empty_channel(), context.output)?;
                last_job.join(&context.printer);
            }
        }
        Ok(())
    }


    fn push_arguments_to_env(mut arguments: Vec<Argument>, env: &Namespace) {
        for arg in arguments.drain(..) {
            if let Some(name) = &arg.name {
                env.declare_str(name.as_ref(), arg.value);
            }
        }
    }
}

impl ToString for Closure {
    fn to_string(&self) -> String {
        self.job_definitions.iter().map(|j| j.to_string()).collect::<Vec<String>>().join("; ")
    }
}
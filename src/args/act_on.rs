//! Specifying the pods, services or both acted on by a command.

use std::fmt;
use std::slice;

use errors::*;
use pod::Pod;
use project::{PodOrService, Project};

/// The names of pods, services or both to pass to one of our commands.
#[derive(Debug)]
pub enum ActOn {
    /// Act upon all the pods and/or services associated with this project.
    All,
    /// Act upon all the pods and/or services associated with this project, in reverse order.
    AllReversed,
    /// Act on services except those defined in `Pod`s of type
    /// `PodType::Task`.
    AllExceptTasks,
    /// Act on services except those defined in `Pod`s of type
    /// `PodType::Task`, reversed.
    AllExceptTasksReversed,
    /// Act upon only the named pods and/or services.
    Named(Vec<String>),
}

impl ActOn {
    /// Iterate over the pods or services specified by this `ActOn` object.
    pub fn pods_or_services<'a>(&'a self, project: &'a Project) -> Result<PodsOrServices<'a>> {
        let state = match *self {
            ActOn::All => State::PodIter(project.ordered_pods()?.into_iter()),
            ActOn::AllReversed => State::PodIter(project.ordered_pods()?.into_iter_reversed()),
            ActOn::AllExceptTasks => State::PodIter(project.ordered_pods()?.into_iter_without_tasks()),
            ActOn::AllExceptTasksReversed => State::PodIter(project.ordered_pods()?.into_iter_without_tasks_reversed()),
            ActOn::Named(ref names) => State::NameIter(names.into_iter()),
        };
        Ok(PodsOrServices {
            project: project,
            state: state,
        })
    }
    /// Reverse this `ActOn` object.
    pub fn reverse(self) -> ActOn {
        match self {
            ActOn::All => ActOn::AllReversed,
            ActOn::AllReversed => ActOn::All,
            ActOn::AllExceptTasks => ActOn::AllExceptTasksReversed,
            ActOn::AllExceptTasksReversed => ActOn::AllExceptTasks,
            x => x,
        }
    }
}

/// Internal state for `PodsOrServices` iterator.
#[cfg_attr(feature = "clippy", allow(enum_variant_names))]
enum State<'a> {
    /// This corresponds to `ActOn::All`.
    PodIter(Box<Iterator<Item = &'a Pod> + 'a >),
    /// This corresponds to `ActOn::Named`.
    NameIter(slice::Iter<'a, String>),
}

impl<'a> fmt::Debug for State<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            State::PodIter(_) => f.write_str("State::PodIter"),
            State::NameIter(_) => f.write_str("State::NameIter"),
        }
    }
}

/// An iterator over the pods or services specified by an `ActOn` value.
#[derive(Debug)]
pub struct PodsOrServices<'a> {
    /// The project with which we're associated.
    project: &'a Project,

    /// Our internal iteration state.
    state: State<'a>,
}

impl<'a> Iterator for PodsOrServices<'a> {
    type Item = Result<PodOrService<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::PodIter(ref mut iter) => {
                iter.next().map(|pod| Ok(PodOrService::Pod(pod)))
            }
            State::NameIter(ref mut iter) => if let Some(name) = iter.next() {
                Some(self.project.pod_or_service_or_err(name))
            } else {
                None
            },
        }
    }
}
